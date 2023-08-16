sudo docker build --platform=linux/amd64 . -t fp-core
sudo docker tag fp-core:latest public.ecr.aws/b3c4u5n1/filecoin-core-api:latest
sudo aws ecr-public get-login-password --region us-east-1 | sudo docker login --username AWS --password-stdin public.ecr.aws
sudo docker push public.ecr.aws/b3c4u5n1/filecoin-core-api:latest

