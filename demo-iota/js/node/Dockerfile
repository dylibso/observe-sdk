FROM node:20.4.0

EXPOSE 3000

ADD /js /js
ADD /demo-iota/js/node /demo-iota/js/node

WORKDIR /demo-iota/js/node
RUN npm i

CMD node index.js
