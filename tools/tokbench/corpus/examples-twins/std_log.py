# Python twin of examples/std_log.ail: leveled messages go to standard error.
import logging
import sys

logging.basicConfig(stream=sys.stderr, level=logging.INFO, format="%(levelname)s %(message)s")
log = logging.getLogger("example")
log.debug("parsing 3 records")
log.info("loaded config")
log.warning("cache is cold")
log.error("disk full, 2 records not written")
print("done")
