import clickhouse_connect
from flask import Flask, jsonify, request
import time
import logging
from urllib3.exceptions import ProtocolError
from clickhouse_connect.driver.exceptions import ClickHouseError
import threading
from flask_cors import CORS

app = Flask(__name__)
# Configure CORS
CORS(app, resources={r"/*": {"origins": "*"}})

@app.before_request
def log_request_info():
    app.logger.debug('Headers: %s', request.headers)
    app.logger.debug('Body: %s', request.get_data())

@app.after_request
def after_request(response):
    # Remove existing CORS headers to prevent duplication
    response.headers.pop('Access-Control-Allow-Origin', None)
    response.headers.pop('Access-Control-Allow-Headers', None)
    response.headers.pop('Access-Control-Allow-Methods', None)

    # Set CORS headers
    response.headers['Access-Control-Allow-Origin'] = 'http://localhost:3000'
    response.headers['Access-Control-Allow-Headers'] = 'Content-Type,Authorization'
    response.headers['Access-Control-Allow-Methods'] = 'GET,PUT,POST,DELETE,OPTIONS'
    
    app.logger.debug('Response Headers: %s', response.headers)
    return response

logging.basicConfig(level=logging.DEBUG)

# Configuration
CLICKHOUSE_HOST = 'REDACTED_CLICKHOUSE_HOST'
CLICKHOUSE_PORT = REDACTED_CLICKHOUSE_PORT
CLICKHOUSE_USER = 'REDACTED_CLICKHOUSE_USER'
CLICKHOUSE_PASSWORD = 'REDACTED_CLICKHOUSE_PASSWORD'
CACHE_TIMEOUT = 300  # Cache timeout in seconds (5 minutes)
MAX_RETRIES = 3
RETRY_DELAY = 5  # seconds
MERGE_BLOCK = 15537393  # The merge block number

pool_addresses = [
    "0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640",
    "0x3416cf6c708da44db2624d63ea0aaef7113527c6",
    "0x11b815efb8f581194ae79006d24e0d814b7697f6",
    "0x4585fe77225b41b697c938b018e2ac67ac5a20c0",
    "0x109830a1aaad605bbf02a9dfa7b0b92ec2fb7daa",
    "0x8ad599c3a0ff1de082011efddc58f1908eb6e6d8",
    "0xc7bbec68d12a0d1830360f8ec58fa599ba1b0e9b",
    "0xcbcdf9626bc03e24f779434178a73a0b4bad62ed",
    "0x5777d92f208679db4b9778590fa3cab3ac9e2168",
    "0x4e68ccd3e89f51c3074ca5072bbac773960dfa36",
    "0x60594a405d53811d3bc4766596efd80fd545a270",
    "0x7858e59e0c01ea06df3af3d20ac7b0003275d4bf",
    "0x435664008F38B0650fBC1C9fc971D0A3Bc2f1e47",
    "0xa6cc3c2531fdaa6ae1a3ca84c2855806728693e8",
    "0x11950d141ecb863f01007add7d1a342041227b58",
    "0x9a772018fbd77fcd2d25657e5c547baff3fd7d16",
    "0x99ac8ca7087fa4a2a1fb6357269965a2014abc35",
    "0xa3f558aebaecaf0e11ca4b2199cc5ed341edfd74",
    "0x1d42064fc4beb5f8aaf85f4617ae8b3b5b8bd801",
    "0xc2e9f25be6257c210d7adf0d4cd6e3e881ba25f8",
    "0x48da0965ab2d2cbf1c17c09cfb5cbe67ad5b1406",
    "0x840deeef2f115cf50da625f7368c24af6fe74410"
]

# Global variables
cached_median_lvr = {}
last_fetch_time = 0
last_queried_block = MERGE_BLOCK  # Initialize with the MERGE_BLOCK
cache_initialized = threading.Event()


def init_client():
    return clickhouse_connect.get_client(host=CLICKHOUSE_HOST, 
                                         port=CLICKHOUSE_PORT, 
                                         user=CLICKHOUSE_USER, 
                                         password=CLICKHOUSE_PASSWORD,
                                         connect_timeout=1200,  # 20 minutes
                                         send_receive_timeout=1200,  # 20 minutes
                                        )



# Enable CORS for all routes as a fallback
@app.after_request
def after_request(response):
    response.headers.add('Access-Control-Allow-Origin', 'http://localhost:3000')
    response.headers.add('Access-Control-Allow-Headers', 'Content-Type,Authorization')
    response.headers.add('Access-Control-Allow-Methods', 'GET,PUT,POST,DELETE,OPTIONS')
    return response

def fetch_median_lvr():
    global last_queried_block
    client = init_client()
    
    query = f"""
        WITH max_block AS (
            SELECT MAX(block_number) AS max_block_num
            FROM brontes.block_analysis
            WHERE block_number >= {last_queried_block}
        )
        SELECT 
            p.profit AS pool_address,
            quantileExact(0.5)(p.profit_amt + p.revenue_amt) AS median_lvr,
            max_block_num
        FROM brontes.block_analysis
        ARRAY JOIN cex_dex_arbed_pool_all AS p
        CROSS JOIN max_block
        WHERE p.profit != '0x0000000000000000000000000000000000000000' AND 
            p.revenue != '0x0000000000000000000000000000000000000000' AND
            p.profit IN {tuple(pool_addresses)} AND
            block_number >= {last_queried_block}
        GROUP BY p.profit, max_block_num
    """
    
    for attempt in range(MAX_RETRIES):
        try:
            results = client.query(query).result_rows
            if results:
                logging.info(f"Query returned {len(results)} results")
                new_data = {pool_address.lower(): median_lvr for pool_address, median_lvr, _ in results}
                last_queried_block = max(result[2] for result in results)  # Update last queried block
                logging.info(f"Updated last_queried_block to {last_queried_block}")
                return new_data
            else:
                logging.warning("Query returned no results")
            return None
        except (ProtocolError, ClickHouseError) as e:
            logging.error(f"Attempt {attempt + 1} failed: {str(e)}")
            if attempt < MAX_RETRIES - 1:
                time.sleep(RETRY_DELAY)
            else:
                raise

def get_median_lvr():
    global cached_median_lvr, last_fetch_time
    
    cache_initialized.wait()  # Wait for cache to be initialized
    
    current_time = time.time()
    if current_time - last_fetch_time > CACHE_TIMEOUT:
        try:
            new_data = fetch_median_lvr()
            if new_data:
                # Update the cache with new data
                cached_median_lvr.update(new_data)
            last_fetch_time = current_time
        except Exception as e:
            logging.error(f"Failed to fetch new data: {str(e)}")
    
    return cached_median_lvr

@app.route('/median_lvr', methods=['GET', 'OPTIONS'])
def get_median_lvr_api():
    if request.method == "OPTIONS":
        return app.make_default_options_response()
    data = get_median_lvr()
    result = [{"pool_address": address, "median_lvr": lvr} for address, lvr in data.items()]
    return jsonify(result)

def initialize_cache():
    global cached_median_lvr, last_queried_block
    logging.info("Initializing cache...")
    
    try:
        initial_data = fetch_median_lvr()
        if initial_data:
            cached_median_lvr = initial_data
            logging.info(f"Cache initialized with {len(cached_median_lvr)} pools")
            logging.info(f"Last queried block: {last_queried_block}")
        else:
            logging.warning("No data fetched during cache initialization")
    except Exception as e:
        logging.error(f"Error during cache initialization: {str(e)}")
    
    cache_initialized.set()  # Signal that cache initialization attempt is complete


def start_flask():
    app.run(debug=True, use_reloader=False, host='0.0.0.0', port=50001)

if __name__ == '__main__':
    init_thread = threading.Thread(target=initialize_cache)
    init_thread.start()
    
    # Wait for cache to be initialized before starting Flask
    cache_initialized.wait()
    
    # Start Flask in the main thread
    start_flask()