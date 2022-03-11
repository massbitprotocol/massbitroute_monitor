#!/bin/bash

rsync -avz ../target/release/mbr-check-component mbr-verify:/opt/verification/mbr-check-component
ssh mbr-verify < restart_verify_service.sh