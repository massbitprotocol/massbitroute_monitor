# MassbitRoute Stat Component

## Install with docker

```
 services:
  stat:
    privileged: true
    restart: unless-stopped
    image: massbit/massbitroute_stat:_BRANCH_
    build:
      context: /massbit/massbitroute/app/src
      dockerfile: install/mbr/build/stat/Dockerfile
      args:
        GIT_PUBLIC_URL: https://github.com
        #MYAPP_IMAGE: massbit/massbitroute_base:_BRANCH_
        BRANCH: _BRANCH_
    container_name: mbr_stat    
    environment:
      - STAT_TYPE=node                                               # stat type (node or gatway)
      - STAT_NETWORK=eth                                             # stat network of blockchain
      - STAT_BLOCKCHAIN=mainnet                                      # stat blockchain name
      - GIT_PUBLIC_URL="https://github.com"                          # default public source control	
      - PORTAL_URL=http://portal.massbitroute.net                    # default url of portal
      - MBR_ENV=_BRANCH_                                             # Git Tag version deployment of Api repo
      - MKAGENT_BRANCH=_BRANCH_                                      # Git Tag version deployment of Monitor client
      - GIT_PRIVATE_BRANCH=_BRANCH_                                  # Private git branch default of runtime conf
      - GIT_PRIVATE_READ_URL=http://massbit:xxx@git.massbitroute.net # Private git url address with authorized account

    extra_hosts:
      - "git.massbitroute.net:172.20.0.2"
      - "api.massbitroute.net:172.20.0.3"      
      - "portal.massbitroute.net:127.0.0.1"
```
