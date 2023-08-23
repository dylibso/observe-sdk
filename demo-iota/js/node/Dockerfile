FROM node:20.4.0

EXPOSE 3000

ADD /js /js
ADD /demo-iota/js/node /demo-iota/js/node

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

WORKDIR /demo-iota/js/node
RUN npm i

CMD node index.js
