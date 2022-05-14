#!/bin/bash
cp ~/.ssh/config-$1 ~/.ssh/config
cp ../.env-$1 ../.env
export ENV=$1