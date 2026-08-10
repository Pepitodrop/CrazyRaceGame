args <- commandArgs(trailingOnly = TRUE)
seed <- if (length(args) >= 1) suppressWarnings(as.integer(args[[1]])) else 0L
output <- if (length(args) >= 2) args[[2]] else "/tmp/crazy-race-track.tsv"

fresh_seed <- function() {
  as.integer((as.numeric(Sys.time()) * 1000) %% .Machine$integer.max)
}

sample_speed <- function(terrain) {
  ranges <- list(
    straight = 0:3,
    curve = -2:1,
    mud = -4:-1,
    jump = 1:3
  )
  sample(ranges[[terrain]], 1)
}

sample_boost <- function(terrain) {
  sample(if (terrain == "jump") 2:4 else 0:2, 1)
}

# Zero means "fresh track for this server start". Positive seeds remain
# reproducible for tests, demos, and rollbacks.
if (is.na(seed) || seed <= 0L) {
  seed <- fresh_seed()
}

set.seed(seed)

segment_count <- 20L
terrain_names <- c("straight", "curve", "mud", "jump")
terrain_weights <- c(0.40, 0.30, 0.17, 0.13)

terrains <- sample(
  terrain_names,
  size = segment_count,
  replace = TRUE,
  prob = terrain_weights
)
terrains[c(1, segment_count)] <- "straight"

track <- data.frame(
  index = 0:(segment_count - 1L),
  terrain = terrains,
  speed = vapply(terrains, sample_speed, integer(1)),
  boost = vapply(terrains, sample_boost, integer(1)),
  recovery = sample(1:4, segment_count, replace = TRUE)
)

write.table(
  track,
  file = output,
  sep = "\t",
  quote = FALSE,
  row.names = FALSE
)
