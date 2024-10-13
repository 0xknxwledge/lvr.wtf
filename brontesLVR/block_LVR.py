import clickhouse_connect
from flask import Flask, jsonify, request
from collections import defaultdict
import time
import logging
from urllib3.exceptions import ProtocolError
from clickhouse_connect.driver.exceptions import ClickHouseError
import threading
from flask_caching import Cache
from flask_cors import CORS

app = Flask(__name__)
cache = Cache(app, config={'CACHE_TYPE': 'simple'})
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
CACHE_TIMEOUT = 60  # Cache timeout in seconds
UPDATE_INTERVAL = 60  # Update interval in seconds
PAGE_SIZE = 100  # Number of rows per page

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
last_update_time = 0
last_queried_block = 0
update_lock = threading.Lock()

def init_client():
    return clickhouse_connect.get_client(host=CLICKHOUSE_HOST, port=CLICKHOUSE_PORT, 
                                         user=CLICKHOUSE_USER, password=CLICKHOUSE_PASSWORD)

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

def update_cache():
    global cached_data, last_update_time, last_queried_block
    with update_lock:
        current_time = time.time()
        if current_time - last_update_time > UPDATE_INTERVAL:
            try:
                new_data = fetch_data(last_queried_block)
                cached_data.update(new_data)
                if new_data:
                    last_queried_block = max(new_data.keys())
                last_update_time = current_time
                logging.info(f"Cache updated with {len(new_data)} new entries. Last queried block: {last_queried_block}")
            except Exception as e:
                logging.error(f"Failed to update cache: {str(e)}")

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

def start_update_thread():
    while True:
        update_cache()
        time.sleep(UPDATE_INTERVAL)

if __name__ == '__main__':
    logging.info("Starting update thread")
    update_thread = threading.Thread(target=start_update_thread)
    update_thread.daemon = True
    update_thread.start()
    
    logging.info("Starting Flask application")
    app.run(debug=True, use_reloader=False, host='127.0.0.1', port=5001) 