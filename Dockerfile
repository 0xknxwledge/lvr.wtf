# Use an official Python runtime as the base image
FROM python:3.11-slim

# Set the working directory in the container
WORKDIR /app

# Copy the application code
COPY brontesLVR /app/brontesLVR

# Install the required packages
RUN pip3 install flask_caching clickhouse_connect flask_cors

# Expose port 50001 for the Flask app
EXPOSE 50001

# Set environment variables
ENV FLASK_ENV=production
ENV CLICKHOUSE_HOST='34.149.107.219'
ENV CLICKHOUSE_PORT='8123'
ENV CLICKHOUSE_USER='john_beecher'
ENV CLICKHOUSE_PASSWORD='dummy-password'

# Run the Flask application
CMD ["python3", "brontesLVR/application.py"]