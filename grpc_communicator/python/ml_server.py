import grpc
import json
from concurrent import futures
import service_pb2
import service_pb2_grpc

class VectorServiceServicer(service_pb2_grpc.VectorServiceServicer):
    def ProcessJson(self, request, context):
        json_data = json.loads(request.json_data)
        # Process the JSON data and generate a vector response
        vector = [1.0, 2.0, 3.0]  # Example vector response
        return service_pb2.VectorResponse(vector=vector)

    def ProcessCircuitFiles(self, request, context):
        el_content = request.el_content
        csv_content = request.csv_content
        json_content = request.json_content

        # Process the files here
        # Graph encoding, pre-processing for the data

        # Placeholder for actual processing
        delay = 1.23  # Replace with inference result from HOGA

        return service_pb2.CircuitProcessingResponse(delay=delay)

def serve():
    server = grpc.server(futures.ThreadPoolExecutor(max_workers=10))
    service_pb2_grpc.add_VectorServiceServicer_to_server(VectorServiceServicer(), server)
    server.add_insecure_port('[::]:50051')
    server.start()
    server.wait_for_termination()

if __name__ == '__main__':
    serve()