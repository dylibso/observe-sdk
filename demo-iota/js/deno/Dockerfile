# FROM denoland/deno:ubuntu-1.36.1
FROM node:20.4.0

EXPOSE 3000

# install curl, so we can install nvm, so we can install node
ENV PATH="${PATH}:/usr/bin"
# ENV NODE_VERSION=20.4.0
RUN apt update
RUN apt install -y curl
RUN curl -fsSL https://deno.land/x/install/install.sh | sh
# RUN apt install -y python3
# RUN apt install -y make
# RUN apt install -y gcc
# RUN curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.4/install.sh | bash

# ENV NVM_DIR=/root/.nvm
# RUN . "$NVM_DIR/nvm.sh" && nvm install ${NODE_VERSION}
# RUN . "$NVM_DIR/nvm.sh" && nvm use v${NODE_VERSION}
# RUN . "$NVM_DIR/nvm.sh" && nvm alias default v${NODE_VERSION}
# ENV PATH="/root/.nvm/versions/node/v${NODE_VERSION}/bin/:${PATH}"
# RUN node --version
# RUN npm --version
RUN echo $PATH
ADD /js /js
ADD /demo-iota/js/deno /demo-iota/js/deno

# install the global build dependencies used for each adapter
WORKDIR /js
RUN npm i

# build the packages for each adapter
WORKDIR /js/packages/observe-sdk-datadog
RUN npm run build

WORKDIR /js/packages/observe-sdk-honeycomb
RUN npm run build

WORKDIR /js/packages/observe-sdk-lightstep
RUN npm run build

WORKDIR /js/packages/observe-sdk-stdout
RUN npm run build

WORKDIR /demo-iota/js/deno

CMD ./run.sh
