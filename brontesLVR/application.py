import clickhouse_connect
from flask import Flask, jsonify, request
from collections import defaultdict
import time
import logging
from urllib3.exceptions import ProtocolError
from clickhouse_connect.driver.exceptions import ClickHouseError
import threading
from flask_cors import CORS
from flask_caching import Cache
import os

app = Flask(__name__)
cache = Cache(app, config={'CACHE_TYPE': 'simple'})
CORS(app, resources={r"/*": {"origins": "*"}})

logging.basicConfig(level=logging.DEBUG)

# Configuration
CLICKHOUSE_HOST = os.environ.get('CLICKHOUSE_HOST', '34.149.107.219')
CLICKHOUSE_PORT = os.environ.get('CLICKHOUSE_PORT', 8123)
CLICKHOUSE_USER = os.environ.get('CLICKHOUSE_USER', 'john_beecher')
CLICKHOUSE_PASSWORD = os.environ.get('CLICKHOUSE_PASSWORD', 'dummy-password')
CACHE_TIMEOUT = 300  # Cache timeout in seconds (5 minutes)
UPDATE_INTERVAL = 60  # Update interval in seconds
PAGE_SIZE = 100  # Number of rows per page
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
cached_data = defaultdict(float)
cached_median_lvr = {}
last_update_time = 0
last_queried_block = MERGE_BLOCK
update_lock = threading.Lock()
cache_initialized = threading.Event()

def init_client():
    return clickhouse_connect.get_client(host=CLICKHOUSE_HOST, 
                                         port=CLICKHOUSE_PORT, 
                                         user=CLICKHOUSE_USER, 
                                         password=CLICKHOUSE_PASSWORD,
                                         connect_timeout=1200,  # 20 minutes
                                         send_receive_timeout=1200)  # 20 minutes

def fetch_data(start_block):
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
            block_number > {start_block}
        GROUP BY block_number
        ORDER BY block_number
    """
    
    logging.info(f"Executing query to fetch data from block {start_block}")
    
    try:
        results = client.query(query).result_rows
        logging.info(f"Query returned {len(results)} rows")
        return {block_number: total_lvr for block_number, total_lvr in results}
    except (ProtocolError, ClickHouseError) as e:
        logging.error(f"Error executing query: {str(e)}")
        return {}

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

def update_cache():
    global cached_data, cached_median_lvr, last_update_time, last_queried_block
    with update_lock:
        current_time = time.time()
        if current_time - last_update_time > UPDATE_INTERVAL:
            try:
                new_data = fetch_data(last_queried_block)
                cached_data.update(new_data)
                if new_data:
                    last_queried_block = max(new_data.keys())
                
                new_median_data = fetch_median_lvr()
                if new_median_data:
                    cached_median_lvr.update(new_median_data)
                
                last_update_time = current_time
                logging.info(f"Cache updated. Last queried block: {last_queried_block}")
            except Exception as e:
                logging.error(f"Failed to update cache: {str(e)}")

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
    update_cache()
    result = calculate_running_total(cached_data)
    return jsonify(result)

@app.route('/lvr_table', methods=['GET', 'OPTIONS'])
def get_lvr_table():
    if request.method == "OPTIONS":
        return app.make_default_options_response()
    
    update_cache()
    
    page = int(request.args.get('page', 1))
    
    with update_lock:
        sorted_data = sorted(cached_data.items(), reverse=True)  # Sort in descending order
    
    total_pages = (len(sorted_data) - 1) // PAGE_SIZE + 1
    
    if page < 1 or page > total_pages:
        return jsonify({"error": "Invalid page number"}), 400
    
    start_index = (page - 1) * PAGE_SIZE
    end_index = start_index + PAGE_SIZE
    
    paginated_data = sorted_data[start_index:end_index]
    
    result = {
        "data": [{"block_number": block, "total_lvr": lvr} for block, lvr in paginated_data],
        "total_pages": total_pages,
        "current_page": page,
        "last_queried_block": last_queried_block
    }
    
    logging.info(f"Returning data for page {page} ({len(paginated_data)} entries)")
    
    return jsonify(result)

@app.route('/median_lvr', methods=['GET', 'OPTIONS'])
def get_median_lvr_api():
    if request.method == "OPTIONS":
        return app.make_default_options_response()
    update_cache()
    result = [{"pool_address": address, "median_lvr": lvr} for address, lvr in cached_median_lvr.items()]
    return jsonify(result)

def initialize_cache():
    global cached_data, cached_median_lvr, last_queried_block
    logging.info("Initializing cache...")
    
    try:
        cached_data = fetch_data(MERGE_BLOCK)
        if cached_data:
            last_queried_block = max(cached_data.keys())
        
        initial_median_data = fetch_median_lvr()
        if initial_median_data:
            cached_median_lvr = initial_median_data
        
        logging.info(f"Cache initialized. Last queried block: {last_queried_block}")
    except Exception as e:
        logging.error(f"Error during cache initialization: {str(e)}")
    
    cache_initialized.set()  # Signal that cache initialization attempt is complete

def start_update_thread():
    while True:
        update_cache()
        time.sleep(UPDATE_INTERVAL)

application = app
if __name__ == '__main__':
    init_thread = threading.Thread(target=initialize_cache)
    init_thread.start()
    
    # Wait for cache to be initialized before starting Flask
    cache_initialized.wait()
    
    # Start the update thread
    update_thread = threading.Thread(target=start_update_thread)
    update_thread.daemon = True
    update_thread.start()
    
    # Start Flask in the main thread
    app.run(debug=True, use_reloader=False, host='0.0.0.0', port=5000)