# FROM denoland/deno:ubuntu-1.36.1
FROM node:20.4.0

EXPOSE 3000

# install curl, so we can install nvm, so we can install node
ENV PATH="${PATH}:/usr/bin"
RUN apt update
RUN apt install -y curl
RUN curl -fsSL https://deno.land/x/install/install.sh | sh
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
