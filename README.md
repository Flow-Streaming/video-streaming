# Video Streaming Microservice

A Rust-based video streaming microservice that uses Supabase for database storage and file storage. Built with Axum for robust, high-performance HTTP request handling.

## Features

- Upload videos to Supabase Storage
- Stream videos from Supabase Storage
- Store video metadata in Supabase Database
- RESTful API endpoints for video management
- Integration with Flutter clients

## Setup

### Prerequisites

- Rust installed (latest stable version)
- A Supabase account and project
- A bucket named 'videos' in your Supabase storage

### Environment Configuration

Update the `.env` file with your Supabase credentials:

```
SUPABASE_URL=https://your-project-id.supabase.co
SUPABASE_API_KEY=your-supabase-api-key
SUPABASE_BUCKET=videos
```

## Building and Running

Build and run the service:

```bash
cargo build --release
./target/release/video-streaming
```

The service will run on port 3000 by default.

## API Endpoints

### List Videos

```
GET /videos
```

Returns a list of all videos with metadata.

### Get Video Details

```
GET /videos/{id}
```

Returns metadata for a specific video.

### Create Video

```
POST /videos
Content-Type: application/json

{
  "title": "My Video",
  "description": "Description of the video"
}
```

Creates a new video entry and returns an upload URL.

### Upload Video

```
POST /videos/{id}/upload
Content-Type: multipart/form-data

Form field: "video" (file)
```

Uploads a video file to Supabase storage.

### Stream Video

```
GET /videos/{id}/stream
```

Returns a direct streaming URL for the video.

### Delete Video

```
DELETE /videos/{id}
```

Deletes a video and its associated file.

## Flutter Client Integration

In your Flutter app, you can use the following code to interact with the API:

```dart
import 'package:http/http.dart' as http;
import 'dart:convert';
import 'package:http_parser/http_parser.dart';

class VideoService {
  final String baseUrl = 'http://your-server:3000';

  Future<List<VideoMetadata>> listVideos() async {
    final response = await http.get(Uri.parse('$baseUrl/videos'));
    if (response.statusCode == 200) {
      final List<dynamic> data = jsonDecode(response.body);
      return data.map((json) => VideoMetadata.fromJson(json)).toList();
    } else {
      throw Exception('Failed to load videos');
    }
  }

  Future<String> createVideo(String title, String? description) async {
    final response = await http.post(
      Uri.parse('$baseUrl/videos'),
      headers: {'Content-Type': 'application/json'},
      body: jsonEncode({
        'title': title,
        'description': description,
      }),
    );

    if (response.statusCode == 200) {
      final data = jsonDecode(response.body);
      return data['id'];
    } else {
      throw Exception('Failed to create video');
    }
  }

  Future<void> uploadVideo(String videoId, File videoFile) async {
    var request = http.MultipartRequest(
      'POST',
      Uri.parse('$baseUrl/videos/$videoId/upload'),
    );

    request.files.add(
      await http.MultipartFile.fromPath(
        'video',
        videoFile.path,
        contentType: MediaType('video', 'mp4'),
      ),
    );

    var response = await request.send();
    if (response.statusCode != 200) {
      throw Exception('Failed to upload video');
    }
  }

  Future<String> getStreamUrl(String videoId) async {
    final response = await http.get(Uri.parse('$baseUrl/videos/$videoId/stream'));
    if (response.statusCode == 200) {
      return jsonDecode(response.body);
    } else {
      throw Exception('Failed to get streaming URL');
    }
  }

  Future<void> deleteVideo(String videoId) async {
    final response = await http.delete(Uri.parse('$baseUrl/videos/$videoId'));
    if (response.statusCode != 204) {
      throw Exception('Failed to delete video');
    }
  }
}

class VideoMetadata {
  final String id;
  final String title;
  final String? description;
  final String streamUrl;
  final String? thumbnailUrl;
  final String createdAt;

  VideoMetadata({
    required this.id,
    required this.title,
    this.description,
    required this.streamUrl,
    this.thumbnailUrl,
    required this.createdAt,
  });

  factory VideoMetadata.fromJson(Map<String, dynamic> json) {
    return VideoMetadata(
      id: json['id'],
      title: json['title'],
      description: json['description'],
      streamUrl: json['stream_url'],
      thumbnailUrl: json['thumbnail_url'],
      createdAt: json['created_at'],
    );
  }
}
```
