use Test::Nginx::Socket::Lua 'no_plan';

repeat_each(1);

no_shuffle();

# plan tests => blocks() * repeat_each() * 2;
$ENV{TEST_NGINX_HTML_DIR} ||= html_dir();
$ENV{TEST_NGINX_BINARY} =
"/massbit/massbitroute/app/src/sites/services/monitor/bin/openresty/nginx/sbin/nginx";
our $main_config = <<'_EOC_';
  load_module /massbit/massbitroute/app/src/sites/services/monitor/bin/openresty/nginx/modules/ngx_http_link_func_module.so;
      load_module /massbit/massbitroute/app/src/sites/services/monitor/bin/openresty/nginx/modules/ngx_http_geoip2_module.so;
      load_module /massbit/massbitroute/app/src/sites/services/monitor/bin/openresty/nginx/modules/ngx_stream_geoip2_module.so;
      load_module /massbit/massbitroute/app/src/sites/services/monitor/bin/openresty/nginx/modules/ngx_http_vhost_traffic_status_module.so;
      load_module /massbit/massbitroute/app/src/sites/services/monitor/bin/openresty/nginx/modules/ngx_http_stream_server_traffic_status_module.so;
      load_module /massbit/massbitroute/app/src/sites/services/monitor/bin/openresty/nginx/modules/ngx_stream_server_traffic_status_module.so;

_EOC_

our $http_config = <<'_EOC_';
  server_tokens off;
    map_hash_max_size 128;
    map_hash_bucket_size 128;
    server_names_hash_bucket_size 128;
    include /massbit/massbitroute/app/src/sites/services/monitor/bin/openresty/nginx/conf/mime.types;
    access_log /massbit/massbitroute/app/src/sites/services/monitor/logs/nginx/nginx-access.log;
    # tmp
    client_body_temp_path /massbit/massbitroute/app/src/sites/services/monitor/tmp/client_body_temp;
    fastcgi_temp_path /massbit/massbitroute/app/src/sites/services/monitor/tmp/fastcgi_temp;
    proxy_temp_path /massbit/massbitroute/app/src/sites/services/monitor/tmp/proxy_temp;
    scgi_temp_path /massbit/massbitroute/app/src/sites/services/monitor/tmp/scgi_temp;
    uwsgi_temp_path /massbit/massbitroute/app/src/sites/services/monitor/tmp/uwsgi_temp;
    lua_package_path '/massbit/massbitroute/app/src/sites/services/monitor/gbc/src/?.lua;/massbit/massbitroute/app/src/sites/services/monitor/lib/?.lua;/massbit/massbitroute/app/src/sites/services/monitor/src/?.lua;/massbit/massbitroute/app/src/sites/services/monitor/sites/../src/?.lua/massbit/massbitroute/app/src/sites/services/monitor/sites/../lib/?.lua;/massbit/massbitroute/app/src/sites/services/monitor/sites/../src/?.lua;/massbit/massbitroute/app/src/sites/services/monitor/bin/openresty/site/lualib/?.lua;;';
    lua_package_cpath '/massbit/massbitroute/app/src/sites/services/monitor/gbc/src/?.so;/massbit/massbitroute/app/src/sites/services/monitor/lib/?.so;/massbit/massbitroute/app/src/sites/services/monitor/src/?.so;/massbit/massbitroute/app/src/sites/services/monitor/sites/../src/?.so/massbit/massbitroute/app/src/sites/services/monitor/sites/../lib/?.so;/massbit/massbitroute/app/src/sites/services/monitor/sites/../src/?.so;/massbit/massbitroute/app/src/sites/services/monitor/bin/openresty/site/lualib/?.so;;';
            resolver 8.8.8.8 ipv6=off;
            variables_hash_bucket_size 512;
            #ssl
            lua_shared_dict auto_ssl 1m;
            lua_shared_dict auto_ssl_settings 64k;

            #lua
            lua_capture_error_log 32m;
            #lua_need_request_body on;
            lua_regex_match_limit 1500;
            lua_check_client_abort on;
            lua_socket_log_errors off;
            lua_shared_dict _GBC_ 1024k;
            lua_code_cache on;
        

#_INCLUDE_SITES_HTTPINIT_
    init_by_lua '\n    
	   require("framework.init")
	   local appKeys = dofile("/massbit/massbitroute/app/src/sites/services/monitor/tmp/app_keys.lua")
	   local globalConfig = dofile("/massbit/massbitroute/app/src/sites/services/monitor/tmp/config.lua")
	   cc.DEBUG = globalConfig.DEBUG
	   local gbc = cc.import("#gbc")
	   cc.exports.nginxBootstrap = gbc.NginxBootstrap:new(appKeys, globalConfig)
        

--_INCLUDE_SITES_LUAINIT_\n    ';
    init_worker_by_lua '\n    

        

--_INCLUDE_SITES_LUAWINIT_\n    ';

map $http_x_forwarded_for $realip {
    ~^(\d+\.\d+\.\d+\.\d+) $1;
    default $remote_addr;
}
_EOC_

our $config = <<'_EOC_';

 location /push {
        proxy_set_header X-Scheme $scheme;
        proxy_set_header X-Script-Name /push;
        proxy_pass http://127.0.0.1:18889;
    }
  location ~ ^.*/check_mk/ {
        more_clear_headers Content-Security-Policy;
        more_clear_headers X-Content-Type-Options;
        proxy_pass http://localhost:8000;
    }
_EOC_
run_tests();

__DATA__

=== Api create new

--- main_config eval: $::main_config
--- http_config eval: $::http_config
--- config eval: $::config
--- request
GET /mbr/check_mk/index.py
--- error_code: 200
--- no_error_log
