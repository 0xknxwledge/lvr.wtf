import clickhouse_connect
from flask import Flask, jsonify, request
from collections import defaultdict
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
CACHE_TIMEOUT = 60  # Cache timeout in seconds for the latest data
BLOCK_RANGE = 100000  # Fixed block range for running total calculation
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
cached_data = {}
last_fetch_time = 0
last_fetched_block = MERGE_BLOCK
cache_initialized = threading.Event()

def init_client():
    return clickhouse_connect.get_client(host=CLICKHOUSE_HOST, port=CLICKHOUSE_PORT, 
                                         user=CLICKHOUSE_USER, password=CLICKHOUSE_PASSWORD,
                                         connect_timeout=600, send_receive_timeout=600)

def fetch_data(start_block=None):
    client = init_client()
    
    query = f"""
        SELECT 
            block_number,
            sum(p.profit_amt + p.revenue_amt) AS total_lvr
        FROM brontes.block_analysis
        ARRAY JOIN cex_dex_arbed_pool_all AS p
        WHERE p.profit != '0x0000000000000000000000000000000000000000' AND 
            p.profit != '' AND 
            p.revenue != '0x0000000000000000000000000000000000000000' AND 
            p.revenue != '' AND
            p.profit IN {tuple(pool_addresses)} AND
            block_number >= {MERGE_BLOCK}
            {f'AND block_number > {start_block}' if start_block else ''}
        GROUP BY block_number
        ORDER BY block_number
    """
    
    logging.info(f"Executing query with start_block: {start_block if start_block else MERGE_BLOCK}")
    
    for attempt in range(MAX_RETRIES):
        try:
            results = client.query(query).result_rows
            new_data = defaultdict(float)
            for block_number, total_lvr in results:
                new_data[block_number] = total_lvr
            logging.info(f"Query returned {len(new_data)} results")
            return new_data
        except (ProtocolError, ClickHouseError) as e:
            logging.error(f"Attempt {attempt + 1} failed: {str(e)}")
            if attempt < MAX_RETRIES - 1:
                time.sleep(RETRY_DELAY)
            else:
                raise

def get_data():
    global cached_data, last_fetch_time, last_fetched_block
    
    cache_initialized.wait()  # Wait for cache to be initialized
    
    current_time = time.time()
    if current_time - last_fetch_time > CACHE_TIMEOUT:
        try:
            new_data = fetch_data(last_fetched_block)
            if new_data:
                cached_data.update(new_data)
                last_fetched_block = max(new_data.keys())
                logging.info(f"Updated last_fetched_block to {last_fetched_block}")
            last_fetch_time = current_time
        except Exception as e:
            logging.error(f"Failed to fetch new data: {str(e)}")
    
    return cached_data

def calculate_running_total(data):
    sorted_blocks = sorted(data.keys())
    first_block = sorted_blocks[0] if sorted_blocks else MERGE_BLOCK
    last_block = sorted_blocks[-1] if sorted_blocks else MERGE_BLOCK
    
    result = []
    running_total = 0
    
    for block in range(first_block, last_block + 1):
        if block in data:
            running_total += data[block]
        
        if (block - MERGE_BLOCK) % BLOCK_RANGE == 0 or block == last_block:
            result.append({
                'block_number': block,
                'running_total': running_total
            })
    
    return result

@app.route('/lvr_running_total', methods=['GET', 'OPTIONS'])
def get_lvr_running_total():
    if request.method == "OPTIONS":
        return app.make_default_options_response()
    data = get_data()
    result = calculate_running_total(data)
    return jsonify(result)

def initialize_cache():
    global cached_data, last_fetched_block
    logging.info("Initializing cache...")
    
    try:
        cached_data = fetch_data()
        if cached_data:
            last_fetched_block = max(cached_data.keys())
            logging.info(f"Cache initialized with {len(cached_data)} blocks")
            logging.info(f"Initial last_fetched_block: {last_fetched_block}")
        else:
            logging.warning("No data fetched during cache initialization")
    except Exception as e:
        logging.error(f"Error during cache initialization: {str(e)}")
    
    cache_initialized.set()  # Signal that cache initialization attempt is complete

def start_flask():
    app.run(debug=True, use_reloader=False, host='127.0.0.1', port=5002)

if __name__ == '__main__':
    init_thread = threading.Thread(target=initialize_cache)
    init_thread.start()
    
    # Wait for cache to be initialized before starting Flask
    cache_initialized.wait()
    
    # Start Flask in the main thread
    start_flask()