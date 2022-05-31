#!/bin/bash
if [ -z "$1" ]
then
    echo "No environment is supply"
    echo "Current env: $ENV"
    echo 'Change env to dev try `. ./change_env.sh dev`'
    echo 'To deploy try `bash deploy.sh $ENV`'
else
    cp ~/.ssh/config-$1 ~/.ssh/config
    cp ../.env-$1 ../.env
    export ENV=$1

    echo "Changed to environment: $ENV"
fi