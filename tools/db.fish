#!/usr/bin/env fish

set REPO_DIR (git rev-parse --show-toplevel)
set CONTAINER_NAME pg-satelit-scheduler
set DB_NAME satelit-scheduler
set DB_PASSWD postgres
set DB_PORT 5432

source $REPO_DIR/tools/docker.fish

function print_usage
  echo "Manipulates PostgreSQL Docker container.

Usage:
  db COMMAND

  COMMAND:
    start   starts new Docker container with PostgreSQL
    attach  starts 'psql' session
    stop    stops PostgreSQL's container"
end

# Prints mount directory for docker container
function prepare_vol
  set mnt_dir $REPO_DIR/.mount
  set db_dir $mnt_dir/db

  if test -d $db_dir
    echo $db_dir
    return
  end

  mkdir -p $db_dir
  if which chattr >/dev/null
    chattr +C $mnt_dir  # disable CoW on btrfs
  end

  echo $db_dir
end
  
# Starts new postgres container
function start_db -a vol_path
  docker run --rm -d \
    -p $DB_PORT:5432 \
    -e POSTGRES_PASSWORD=$DB_PASSWD \
    --name $CONTAINER_NAME \
    --mount type=bind,source=$vol_path,target=/var/lib/postgresql/data \
    postgres; or exit $status

  echo "DB started in container: $CONTAINER_NAME" >&2
  echo "DB URL: postgresql://postgres:$DB_PASSWD@localhost:$DB_PORT/$DB_NAME"
end

# Attaches to db in container and starts `psql`
function attach_db
  set cmd psql -h localhost \
      -p $DB_PORT \
      -U postgres \
      -d $DB_PASSWD

  docker exec -it $CONTAINER_NAME \
     $cmd; or exit $status
end

# Stops db container
function stop_db
  docker stop $CONTAINER_NAME
end

function main
  set subcmd $argv[1]
  switch "$subcmd"
    case start
      set vol (prepare_vol)
      start_db $vol
    case attach
      attach_db
    case stop
      stop_db
    case --help -h
      print_usage
    case '*'
      echo "Unknown command $subcmd"
      echo "Try '--help' for help"
      exit 1
  end
end

main $argv
