FROM denoland/deno:1.36.1

EXPOSE 3000

ADD /js /js
ADD /demo-iota/js/deno /demo-iota/js/deno

WORKDIR /demo-iota/js/deno

CMD ./run.sh
