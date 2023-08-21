deno run --allow-net --allow-write --allow-env --allow-read index.ts

# Wait for any process to exit
wait -n

# Exit with status of process that exited first
exit $?