import clickhouse_connect
import csv

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

def init_client():
    client = clickhouse_connect.get_client(host='clickhouse.sorella-beechit.com', port=84, user='john_beecher', password='dummy-password')
    return client

if __name__ == "__main__": 
    client = init_client()
    
    # Modified query to include only the specified pool addresses
    query = f"""
        SELECT 
            p.profit AS pool_address,
            sum(p.profit_amt + p.revenue_amt) AS lvr_extracted
        FROM brontes.block_analysis
        ARRAY JOIN cex_dex_arbed_pool_all AS p
        WHERE run_id = 42069 AND
            p.profit != '0x0000000000000000000000000000000000000000' AND 
            p.profit != '' AND 
            p.revenue != '0x0000000000000000000000000000000000000000' AND 
            p.revenue != '' AND
            p.profit IN {tuple(pool_addresses)}
        GROUP BY pool_address
        ORDER BY lvr_extracted DESC
        """
        
    results = client.query(query).result_rows
    
    fields = ['pool_address', 'lvr_extracted']
        
    filtered_results = [list(r) for r in results if r[0].lower() in [addr.lower() for addr in pool_addresses]]

    with open('brontes_Total_LVR.csv', 'w', newline='') as f:
        writer = csv.writer(f)
        writer.writerow(fields)
        writer.writerows(filtered_results)

    print(f"CSV file 'brontes_Total_LVR.csv' has been created with {len(filtered_results)} rows of data.")