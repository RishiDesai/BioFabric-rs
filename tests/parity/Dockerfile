# Dockerfile for BioFabric golden file generation.
#
# Uses Eclipse Temurin JDK 11 to compile BioFabric (Java 1.5 source, compiled
# as Java 8) and the GoldenGenerator harness. Produces golden .noa/.eda/.bif
# files that the Rust parity tests compare against.
#
# Build:
#   docker build -t biofabric-golden -f tests/parity/Dockerfile .
#
# Run (from repo root):
#   docker run --rm \
#     -v $(pwd)/tests/parity/networks/sif:/networks:ro \
#     -v $(pwd)/tests/parity/goldens:/goldens \
#     biofabric-golden /networks/triangle.sif /goldens/triangle_default

FROM eclipse-temurin:11-jdk

WORKDIR /build

# Copy BioFabric source (with headless fixes applied directly)
COPY BioFabric/src /build/biofabric-src

# Copy golden generator source
COPY tests/parity/java-harness/src/GoldenGenerator.java /build/harness/GoldenGenerator.java

# Compile BioFabric source (no external deps needed).
# We use -source 8 / -target 8 since JDK 11 dropped source 5 support.
# BioFabric's code is fully compatible with source 8.
RUN mkdir -p /build/classes && \
    find /build/biofabric-src -name '*.java' > /tmp/sources.txt && \
    javac -source 8 -target 8 \
          -d /build/classes \
          -Xlint:none \
          @/tmp/sources.txt && \
    echo "BioFabric compiled successfully."

# Copy resource files (properties, images, plugin listing) into classes
RUN cd /build/biofabric-src && \
    find . -name '*.properties' -o -name '*.gif' -o -name '*.xml' -o -name '*.html' \
    | while read f; do \
        dir=$(dirname "$f"); \
        mkdir -p "/build/classes/$dir"; \
        cp "$f" "/build/classes/$dir/"; \
      done && \
    echo "Resources copied."

# Compile the golden generator against BioFabric classes
RUN javac -source 8 -target 8 \
          -cp /build/classes \
          -d /build/classes \
          -Xlint:none \
          /build/harness/GoldenGenerator.java && \
    echo "GoldenGenerator compiled successfully."

# Default entrypoint: run the golden generator
ENTRYPOINT ["java", "-Djava.awt.headless=true", "-Xmx4g", "-cp", "/build/classes", "GoldenGenerator"]
