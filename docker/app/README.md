# App

Docker image to run `satelit-scheduler` service in production.

## Building

To build new version of the image run the following command:

``` sh
VERSION="<version>"
docker build -t satelit/satelit-scheduler:"$VERSION" -f docker/app/Dockerfile .
docker push satelit/satelit-scheduler:"$VERSION"
```
