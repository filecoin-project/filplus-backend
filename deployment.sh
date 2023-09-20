# you need to authenticate with AWS first
# by running aws configure
# and then run this script
sudo docker build --platform=linux/amd64 . -t fp-core # build the image
sudo docker tag fp-core:latest public.ecr.aws/b3c4u5n1/filecoin-core-api:latest # tag the image
sudo aws ecr-public get-login-password --region us-east-1 | sudo docker login --username AWS --password-stdin public.ecr.aws # login to the registry
sudo docker push public.ecr.aws/b3c4u5n1/filecoin-core-api:latest # push the image to the registry

