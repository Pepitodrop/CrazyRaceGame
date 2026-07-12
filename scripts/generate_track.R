args <- commandArgs(trailingOnly = TRUE)
seed <- if (length(args) >= 1) suppressWarnings(as.integer(args[[1]])) else 0L
output <- if (length(args) >= 2) args[[2]] else "/tmp/crazy-race-track.tsv"

# A seed of 0 means "fresh track for this server start". Any positive seed
# remains reproducible for tests, demos, and production rollbacks.
if (is.na(seed) || seed <= 0L) {
  seed <- as.integer((as.numeric(Sys.time()) * 1000) %% .Machine$integer.max)
}

set.seed(seed)
segment_count <- 20L
terrains <- sample(
  c("straight", "curve", "mud", "jump"),
  size = segment_count,
  replace = TRUE,
  prob = c(0.40, 0.30, 0.17, 0.13)
)
terrains[[1]] <- "straight"
terrains[[segment_count]] <- "straight"

speed <- vapply(terrains, function(terrain) {
  switch(
    terrain,
    straight = sample(0:3, 1),
    curve = sample(-2:1, 1),
    mud = sample(-4:-1, 1),
    jump = sample(1:3, 1),
    0L
  )
}, integer(1))

boost <- vapply(terrains, function(terrain) {
  if (terrain == "jump") sample(2:4, 1) else sample(0:2, 1)
}, integer(1))

recovery <- sample(1:4, segment_count, replace = TRUE)
track <- data.frame(
  index = 0:(segment_count - 1L),
  terrain = terrains,
  speed = speed,
  boost = boost,
  recovery = recovery
)

write.table(
  track,
  file = output,
  sep = "\t",
  quote = FALSE,
  row.names = FALSE
)
