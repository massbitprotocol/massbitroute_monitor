
variable "project_prefix" {
  type        = string
  description = "The project prefix (mbr)."
}
variable "environment" {
  type        = string
  description = "Environment: dev, test..."
}
variable "default_zone" {
  type = string
}
variable "network_interface" {
  type = string
}
variable "email" {
  type = string
}
variable "map_machine_types" {
  type = map
}


resource "google_compute_instance_from_machine_image" "mbr-core-new-commit-test" {
  name         = "${var.project_prefix}-${var.environment}-core-test-55147"
  machine_type = var.map_machine_types["mbr-core"]
  zone         = "${var.default_zone}"
  provider = google-beta

  tags = ["http-server", "https-server"]
  source_machine_image = "projects/mbr-test-341307/global/machineImages/mbr-test-core"



  network_interface {
    network = "${var.network_interface}"

    access_config {
      // Ephemeral IP
    }
  }

  metadata_startup_script =  <<EOH
  #!/bin/bash
  #-------------------------------------------
  #  Update host file for Massbitroute.dev
  #-------------------------------------------
  sudo sed 's/#DNSStubListener=yes/DNSStubListener=no/' /etc/systemd/resolved.conf -i
  sudo pkill -f systemd-resolve
  sudo systemctl stop systemd-resolved
  sudo systemctl disable systemd-resolved 
  sudo echo nameserver 8.8.8.8 >/etc/resolv.conf
  
  sudo echo "curl https://raw.githubusercontent.com/hoanito/hosts/main/test-hosts-file > /etc/hosts" >> /opt/update_hosts.sh
  sudo chmod 770 /opt/update_hosts.sh

  sudo echo "* * * * * root sudo bash /opt/update_hosts.sh" >> /etc/crontab
  sudo echo "* * * * * sudo bash /opt/update_hosts.sh" > update_hosts.cron
  sudo crontab update_hosts.cron
  sudo service cron reload
  #-------------------------------------------
  #  Install massbitroute core components
  #-------------------------------------------
  sleep 30
  export API_BRANCH=dev
  export GIT_PRIVATE_READ_URL="http://massbit:c671e4ea06280e7a3f6f9aea6e8155fcde9bc703@git.massbitroute.dev"
  git -C /massbit/massbitroute/app/src pull
  cd /massbit/massbitroute/app/src/scripts
  bash run _refresh >> /home/refresh.log
  EOH


  service_account {
    # Google recommends custom service.massbitroute.devccounts that have cloud-platform scope.massbitroute.devnd permissions granted via IAM Roles.
    email = "${var.email}"
    scopes = ["cloud-platform"]
  }

}

output "mbr_core_public_ip" {
  description = "Public IP of new.massbitroute.devPI VM"
  value = google_compute_instance_from_machine_image.mbr-core-new-commit-test.network_interface.0.access_config.0.nat_ip
}


resource "google_compute_instance" "mbr-portal-new-commit-test" {
  name         = "${var.project_prefix}-${var.environment}-portal-test-55147"
  machine_type = var.map_machine_types["mbr-core"]
  zone         = "${var.default_zone}"
  provider = google-beta

  tags = ["http-server", "https-server"]

  boot_disk {
    initialize_params {
      image = "projects/ubuntu-os-cloud/global/images/ubuntu-2004-focal-v20210720"
      size = 20
    }
  }

  network_interface {
    network = "${var.network_interface}"

    access_config {
      nat_ip = "34.101.57.212"
    }
  }

  metadata_startup_script =  <<EOH
  #!/bin/bash
  #-------------------------------------------
  #  Update host file for Massbitroute.dev
  #-------------------------------------------
  sudo echo "curl https://raw.githubusercontent.com/hoanito/hosts/main/test-hosts-file > /etc/hosts" > /opt/update_hosts.sh
  sudo chmod 770 /opt/update_hosts.sh

  sudo echo "* * * * * root sudo bash /opt/update_hosts.sh" >> /etc/crontab
  sudo echo "* * * * * sudo bash /opt/update_hosts.sh" > update_hosts.cron
  sudo crontab update_hosts.cron
  sudo service cron reload

  sudo sed 's/#DNSStubListener=yes/DNSStubListener=no/' /etc/systemd/resolved.conf -i
  sudo pkill -f systemd-resolve

  sudo systemctl stop systemd-resolved
  sudo systemctl disable systemd-resolved 
  sudo echo nameserver 8.8.8.8 >/etc/resolv.conf

  #-------------------------------------------
  #  Install SSH keys
  #-------------------------------------------
  sudo mkdir /opt/ssh-key
  sudo git clone http://ssh:1fdaf5d506fda56b4a50a5e3f24d68799e33cdd2@git.massbitroute.dev/massbitroute/ssh.git -b main /opt/ssh-key
  sudo cp /opt/ssh-key/id_rsa*  /root/.ssh/
  sudo chmod og-rwx /root/.ssh/id_rsa
  sudo cat /opt/ssh-key/ci-ssh-key.pub >> /home/hoang/.ssh/authorized_keys
  
  ssh-keyscan github.com >> ~/.ssh/known_hosts
  
  #-------------------------------------------
  #  Install massbitroute PORTAL components
  #-------------------------------------------
  export PRIVATE_GIT_DOMAIN='git.massbitroute.dev'
  export PRIVATE_GIT_READ_PASSWORD='c671e4ea06280e7a3f6f9aea6e8155fcde9bc703'
  export PRIVATE_GIT_READ_USERNAME='massbit'
  export PRIVATE_GIT_SSL_USERNAME='ssl'
  export PRIVATE_GIT_SSL_PASSWORD='77842e9d937e34029005ca9d90c6f1d09c39b09f'
  export PRIVATE_GIT_SSH_USERNAME='ssh'
  export PRIVATE_GIT_SSH_PASSWORD='1fdaf5d506fda56b4a50a5e3f24d68799e33cdd2'

  #-------------------------------------------
  #  Install packages
  #-------------------------------------------
  sudo apt update
  sudo apt install redis-server npm -y
  sudo systemctl enable redis-server
  sudo npm install --global yarn
  sudo npm cache clean -f
  sudo npm install -g n
  sudo n stable
  sudo yarn global add pm2
  
  #-------------------------------------------
  #  Install PORTAL API
  #-------------------------------------------
  sudo mkdir /opt/user-management
  sudo git clone git@github.com:massbitprotocol/user-management.git -b staging /opt/user-management
  cd /opt/user-management
  cp .env.admin.sample .env.admin
  cp .env.api.sample .env.api
  cp .env.worker.sample .env.worker
  sudo yarn
  sudo yarn build
  sudo pm2 start

  #-------------------------------------------
  #  Install STAKING API
  #-------------------------------------------
  sudo mkdir /opt/test-massbit-staking
  sudo git clone git@github.com:mison201/test-massbit-staking.git  /opt/test-massbit-staking
  cd /opt/test-massbit-staking
  cp .env.sample .env
  sudo yarn
  sudo yarn build
  sudo pm2 start

  #-------------------------------------------
  #  Install NGINX
  #-------------------------------------------
  sudo mkdir -p /opt/ssl
  git clone http://$PRIVATE_GIT_SSL_USERNAME:$PRIVATE_GIT_SSL_PASSWORD@$PRIVATE_GIT_DOMAIN/massbitroute/ssl.git /etc/letsencrypt
  sudo apt update && sudo apt upgrade -y && sudo apt install curl nginx -y
  sudo apt install software-properties-common
  cp /opt/ssh-key/portal.nginx /etc/nginx/sites-enabled/portal
  cp /opt/ssh-key/staking.nginx /etc/nginx/sites-enabled/staking
  sudo service nginx reload

  EOH


  service_account {
    # Google recommends custom service.massbitroute.devccounts that have cloud-platform scope.massbitroute.devnd permissions granted via IAM Roles.
    email = "${var.email}"
    scopes = ["cloud-platform"]
  }

}

output "mbr_portal_public_ip" {
  description = "Public IP of new.massbitroute.devPI VM"
  value = google_compute_instance.mbr-portal-new-commit-test.network_interface.0.access_config.0.nat_ip
}


resource "google_compute_instance" "mbr-rust-new-commit-test" {
  name         = "${var.project_prefix}-${var.environment}-rust-test-55147"
  machine_type = var.map_machine_types["mbr-core"]
  zone         = "${var.default_zone}"
  provider = google-beta

  tags = ["http-server", "https-server"]

  boot_disk {
    initialize_params {
      image = "projects/ubuntu-os-cloud/global/images/ubuntu-2004-focal-v20210720"
      size = 20
    }
  }

  network_interface {
    network = "${var.network_interface}"

    access_config {
      // Ephemeral IP
    }
  }

  metadata_startup_script =  <<EOH
  #!/bin/bash
  #-------------------------------------------
  #  Update host file for Massbitroute.dev
  #-------------------------------------------
  sudo echo "curl https://raw.githubusercontent.com/hoanito/hosts/main/test-hosts-file > /etc/hosts" >> /opt/update_hosts.sh
  sudo chmod 770 /opt/update_hosts.sh

  sudo echo "* * * * * root sudo bash /opt/update_hosts.sh" >> /etc/crontab
  sudo echo "* * * * * sudo bash /opt/update_hosts.sh" > update_hosts.cron
  sudo crontab update_hosts.cron
  sudo service cron reload

  sudo sed 's/#DNSStubListener=yes/DNSStubListener=no/' /etc/systemd/resolved.conf -i
  sudo pkill -f systemd-resolve
  sudo systemctl stop systemd-resolved
  sudo systemctl disable systemd-resolved 
  sudo echo nameserver 8.8.8.8 >/etc/resolv.conf
  
  #-------------------------------------------
  #  Install SSH keys
  #-------------------------------------------
  sudo mkdir /opt/ssh-key
  sudo git clone http://ssh:1fdaf5d506fda56b4a50a5e3f24d68799e33cdd2@git.massbitroute.dev/massbitroute/ssh.git -b main /opt/ssh-key
  sudo cp /opt/ssh-key/id_rsa*  /root/.ssh/
  sudo chmod og-rwx /root/.ssh/id_rsa
  sudo cat /opt/ssh-key/ci-ssh-key.pub >> /home/hoang/.ssh/authorized_keys
  
  ssh-keyscan github.com >> ~/.ssh/known_hosts

  #-------------------------------------------
  #  Install RUST
  #-------------------------------------------
  sudo apt update -y
  curl https://sh.rustup.rs -sSf | sh -s -- -y
  rustup update nightly
  rustup update stable
  rustup target add wasm32-unknown-unknown --toolchain nightly
  sudo apt install build-essential pkg-config libssl-dev supervisor -y

  #-------------------------------------------
  #  Set up supervisor
  #-------------------------------------------
  echo "[program:verification]
command=bash /opt/verification/run.sh
autostart=true
autorestart=true
stderr_logfile=/var/log/verification.err.log
stdout_logfile=/var/log/verification.out.log
user=root
stopasgroup=true" > /etc/supervisor/conf.d/verification.conf

  #-------------------------------------------
  #  Install NGINX
  #-------------------------------------------
  sudo apt install curl nginx -y
  sudo apt install software-properties-common
  sudo curl https://raw.githubusercontent.com/hoanito/hosts/main/verification.nginx > /etc/nginx/sites-enabled/verification

  sudo service nginx restart

  #-------------------------------------------
  #  Install VERIFICATION (RUST)
  #-------------------------------------------
  sudo mkdir -p /massbit/massbitroute/app/src/sites/services/monitor
  sudo git clone https://github.com/massbitprotocol/massbitroute_monitor.git -b dev /massbit/massbitroute/app/src/sites/services/monitor
  cd /massbit/massbitroute/app/src/sites/services/monitor/check_component
  /root/.cargo/bin/cargo build --release
  mkdir -p /opt/verification
  sudo cp target/release/mbr-check-component /opt/verification/mbr-check-component
  sudo cp src/archive/check-flow.json /opt/verification/check-flow.json
  sudo cp /opt/ssh-key/base-endpoint.json  /opt/verification/base-endpoint.json
  sudo cp script/run.sh /opt/verification/run.sh
  sudo cp config_check_component_test.json /opt/verification/config_check_component.json
  
  sudo chmod 770 /opt/verification/run.sh
  sudo supervisorctl reread
  sudo  supervisorctl update
  sudo supervisorctl start verification
  EOH


  service_account {
    # Google recommends custom service.massbitroute.devccounts that have cloud-platform scope.massbitroute.devnd permissions granted via IAM Roles.
    email = "${var.email}"
    scopes = ["cloud-platform"]
  }

}

output "mbr_rust_public_ip" {
  description = "Public IP of new.massbitroute.devPI VM"
  value = google_compute_instance.mbr-rust-new-commit-test.network_interface.0.access_config.0.nat_ip
}

resource "google_compute_instance" "gateway-a140fb52-db0f-4195-af60-0b2dcf6867bd" {
  name         = "${var.project_prefix}-${var.environment}-gateway-eth-a140fb52-db0f-4195-af60-0b2dcf6867bd"
  machine_type = var.map_machine_types["gateway"]
  zone         = "${var.default_zone}"
  provider = google-beta

  tags = ["http-server", "https-server"]

  boot_disk {
    initialize_params {
      image = "projects/ubuntu-os-cloud/global/images/ubuntu-2004-focal-v20210720"
      size = 20
    }
  }

  network_interface {
    network = "${var.network_interface}"

    access_config {
      // Ephemeral IP
    }
  }

  metadata_startup_script =  <<EOH
  #!/bin/bash
  #-------------------------------------------
  #  Update host file for Massbitroute.dev
  #-------------------------------------------
  sudo echo '
  34.101.146.131 hostmaster.massbitroute.dev
  34.101.146.131 ns1.massbitroute.dev
  34.101.146.131 ns2.massbitroute.dev
  34.101.146.131 massbitroute.devdmin.massbitroute.dev
  34.101.146.131 dapi.massbitroute.dev
  34.101.146.131 massbitroute.devpi.massbitroute.dev
  34.101.146.131 dev.massbitroute.dev
  34.101.146.131 staging.massbitroute.dev
  34.101.146.131 production.massbitroute.dev
  34.101.146.131 status.dapi.massbitroute.dev
  34.101.146.131 session.mbr.massbitroute.dev
  34.101.146.131 monitor.mbr.massbitroute.dev
  34.101.146.131 glmr1.bc.massbitroute.dev
  34.101.146.131 dot1.bc.massbitroute.dev
  34.101.146.131 chain.massbitroute.dev
  
  34.101.57.212 portal.massbitroute.dev
  34.101.57.212 staking.massbitroute.dev
  
  34.101.193.8 verify-as.massbitroute.dev
  34.101.193.8 verify.massbitroute.dev
  34.101.193.8 fisherman.massbitroute.dev
  34.101.193.8 stat.mbr.massbitroute.dev
  127.0.0.1 localhost' >> /etc/hosts


  #-------------------------------------------
  #  Install script for GATEWAY
  #-------------------------------------------
  sudo bash -c "$(curl -sSfL 'https://dapi.massbitroute.dev/api/v1/gateway_install?id=a140fb52-db0f-4195-af60-0b2dcf6867bd&user_id=772efcb1-f14e-4e7b-a5a7-d17d97fff8e5&blockchain=eth&network=mainnet&zone=AS&app_key=R1crX2k84NLVGBuqEp2Kvg&portal_url=https://portal.massbitroute.dev&env=dev')"  >> /home/verification.log
  curl -sSfL 'https://dapi.massbitroute.dev/api/v1/gateway_install?id=a140fb52-db0f-4195-af60-0b2dcf6867bd&user_id=772efcb1-f14e-4e7b-a5a7-d17d97fff8e5&blockchain=eth&network=mainnet&zone=AS&app_key=R1crX2k84NLVGBuqEp2Kvg&portal_url=https://portal.massbitroute.dev&env=dev' > /home/install.sh
  EOH

  service_account {
    # Google recommends custom service accounts that have cloud-platform scope and permissions granted via IAM Roles.
    email = "${var.email}"
    scopes = ["cloud-platform"]
  }
}


resource "google_compute_instance" "node-b9774078-4b80-492f-bad6-ab54d8411eeb" {
  name         = "${var.project_prefix}-${var.environment}-node-eth-b9774078-4b80-492f-bad6-ab54d8411eeb"
  machine_type = var.map_machine_types["gateway"]
  zone         = "${var.default_zone}"
  provider = google-beta

  tags = ["http-server", "https-server", "node", "ethereum"]

  boot_disk {
    initialize_params {
      image = "projects/ubuntu-os-cloud/global/images/ubuntu-2004-focal-v20210720"
      size = 20
    }
  }

  network_interface {
    network = "${var.network_interface}"

    access_config {
      // Ephemeral IP
    }
  }

  metadata_startup_script =  <<EOH
  #!/bin/bash
  #-------------------------------------------
  #  Update host file for Massbitroute.dev
  #-------------------------------------------
  sudo echo '
  34.101.146.131 hostmaster.massbitroute.dev
  34.101.146.131 ns1.massbitroute.dev
  34.101.146.131 ns2.massbitroute.dev
  34.101.146.131 massbitroute.devdmin.massbitroute.dev
  34.101.146.131 dapi.massbitroute.dev
  34.101.146.131 massbitroute.devpi.massbitroute.dev
  34.101.146.131 dev.massbitroute.dev
  34.101.146.131 staging.massbitroute.dev
  34.101.146.131 production.massbitroute.dev
  34.101.146.131 status.dapi.massbitroute.dev
  34.101.146.131 session.mbr.massbitroute.dev
  34.101.146.131 monitor.mbr.massbitroute.dev
  34.101.146.131 glmr1.bc.massbitroute.dev
  34.101.146.131 dot1.bc.massbitroute.dev
  34.101.146.131 chain.massbitroute.dev
  
  34.101.57.212 portal.massbitroute.dev
  34.101.57.212 staking.massbitroute.dev
  
  34.101.193.8 verify-as.massbitroute.dev
  34.101.193.8 verify.massbitroute.dev
  34.101.193.8 fisherman.massbitroute.dev
  34.101.193.8 stat.mbr.massbitroute.dev
  127.0.0.1 localhost' >> /etc/hosts


  #-------------------------------------------
  #  Install script for NODE
  #-------------------------------------------
  sudo bash -c "$(curl -sSfL 'https://dapi.massbitroute.dev/api/v1/node_install?id=b9774078-4b80-492f-bad6-ab54d8411eeb&user_id=772efcb1-f14e-4e7b-a5a7-d17d97fff8e5&blockchain=eth&network=mainnet&zone=AS&data_url=http://34.87.241.136:8545&app_key=9GKlJxDA1C2zwAVkypbgqA&portal_url=https://portal.massbitroute.dev&env=dev')"  >> /home/verification.log
  curl -sSfL 'https://dapi.massbitroute.dev/api/v1/node_install?id=b9774078-4b80-492f-bad6-ab54d8411eeb&user_id=772efcb1-f14e-4e7b-a5a7-d17d97fff8e5&blockchain=eth&network=mainnet&zone=AS&data_url=http://34.87.241.136:8545&app_key=9GKlJxDA1C2zwAVkypbgqA&portal_url=https://portal.massbitroute.dev&env=dev' > /home/install.sh
  EOH

  service_account {
    # Google recommends custom service accounts that have cloud-platform scope and permissions granted via IAM Roles.
    email = "${var.email}"
    scopes = ["cloud-platform"]
  }
}
