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
    supervisor1 = [[

[program:monitor_refresh]
command=/bin/bash _SITE_ROOT_/scripts/run loop _refresh_monitors
autorestart=true
redirect_stderr=true
stdout_logfile=_SITE_ROOT_/logs/monitor_refresh.log
]],
    supervisor = [[
[program:check_mk_dapi_collect]
command=/bin/bash _SITE_ROOT_/scripts/checkmk/dapi loop _collect
autorestart=true
redirect_stderr=true
stdout_logfile=_SITE_ROOT_/logs/check_mk_dapi_collect.log

[program:monitor_client]
command=/bin/bash _SITE_ROOT_/etc/mkagent/agents/push.sh _SITE_ROOT_
autorestart=true
redirect_stderr=true
stdout_logfile=_SITE_ROOT_/logs/monitor_client.log

[program:monitor_server]
command=/bin/bash _SITE_ROOT_/scripts/server.sh
autorestart=true
redirect_stderr=true
stdout_logfile=_SITE_ROOT_/logs/monitor_server.log

    ]]
}
return _config
