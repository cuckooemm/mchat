#!/usr/bin/env bash
PNAME=$(grep name Cargo.toml | head -n 1 | awk -F ' = ' '{print$2}' | sed 's/\"//g')
VERSION=$(grep version Cargo.toml | head -n 1 | awk -F ' = ' '{print$2}' | sed 's/\"//g')
DOCKER_REPO="cuckooemm"

if [ x"$1" = "xtest" ]; then
    VERSION=$VERSION-test
fi

IMGNAME="${PNAME}:${VERSION}"
echo "start build ${IMGNAME}..."

docker build -t $PNAME .

if [ $? -ne 0 ];then
    echo "failed to build ${PNAME}."
    exit 1
fi

docker tag $PNAME:latest ${IMGNAME}

# push
echo "push image ${IMGNAME} to ${DOCKER_REPO}"
docker tag ${IMGNAME} ${DOCKER_REPO}/${IMGNAME}

succ=0
for i in `seq 1 3`; do
	docker push ${DOCKER_REPO}/${IMGNAME}
	if [ $? != 0 ];then
		echo "[$i times] failed to docker push"
		echo "retrying to push ${IMGNAME} to ${DOCKER_REPO}/${IMGNAME}..."
	else
		echo "[$i times] success to push image to repo ${DOCKER_REPO}"
		succ=1
		break
	fi
done

if [ $succ == 0 ]; then
	echo "failed to push docker images ${DOCKER_REPO}/${IMGNAME} at last..., exit now."
	exit 1
fi