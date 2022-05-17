local _config = {
    server = {
        nginx = {
            port = "80",
            port_ssl = "443",
            server_name = "massbitroute"
        }
    },
    templates = {},
    apps = {},
    supervisors = {
        ["monitor_client"] = [[
[program:monitor_client]
command=/bin/bash _SITE_ROOT_/../mkagent/agents/push.sh _SITE_ROOT_
autorestart=true
redirect_stderr=true
stopasgroup=true
killasgroup=true
stopsignal=INT
stdout_logfile=_SITE_ROOT_/../mkagent/logs/monitor_client.log
    ]]
    },
    supervisor_tmp = [[
[program:monitor_discover_dapi]
command=/bin/bash _SITE_ROOT_/scripts/checkmk/dapi loop _collect _SITE_ROOT_
autorestart=true
redirect_stderr=true
stopasgroup=true
killasgroup=true
stopsignal=INT
stdout_logfile=_SITE_ROOT_/logs/monitor_discover_dapi.log

]],
    supervisor = [[
[program:monitor_discover_gateway]
command=/bin/bash _SITE_ROOT_/scripts/run _loop 60 _discover_host gateway
autorestart=true
redirect_stderr=true
stopasgroup=true
killasgroup=true
stopsignal=INT
stdout_logfile=_SITE_ROOT_/logs/monitor_discover_gateway.log

[program:monitor_discover_node]
command=/bin/bash _SITE_ROOT_/scripts/run _loop 60 _discover_host node
autorestart=true
redirect_stderr=true
stopasgroup=true
killasgroup=true
stopsignal=INT
stdout_logfile=_SITE_ROOT_/logs/monitor_discover_node.log


[program:monitor_server]
command=/bin/bash _SITE_ROOT_/scripts/server.sh _SITE_ROOT_
autorestart=true
redirect_stderr=true
stopasgroup=true
killasgroup=true
stopsignal=INT
stdout_logfile=_SITE_ROOT_/logs/monitor_server.log

    ]]
}
return _config
