import 'package:flutter/material.dart';
import 'package:tma/src/rust/api/audio.dart';
import 'package:tma/src/rust/frb_generated.dart';
import 'dart:developer' as developer;

Future<void> main() async {
  developer.log('App start');
  WidgetsFlutterBinding.ensureInitialized();
  await RustLib.init();
  try {
    initPlayer();
  } catch (e) {
    developer.log(e.toString());
  }
  developer.log('MyApp');
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      home: Scaffold(
        appBar: AppBar(title: const Text('flutter_rust_bridge quickstart')),
        body: Column(
          children: [
            Center(
              child: TextButton(
                onPressed: () async {
                  try {
                    await loadAudioSourceAndPlay(url: '', jwtValue: '');
                  } catch (e) {
                    debugPrint(e.toString());
                  }
                },
                child: Text('Load and play music'),
              ),
            ),
            Center(
              child: TextButton(
                onPressed: () {
                  final isPaused = playOrPauseAudio();
                  if (isPaused) {
                    debugPrint("Playing");
                  } else {
                    debugPrint("Paused");
                  }
                },
                child: Text('Play or pause music'),
              ),
            ),
            Center(
              child: TextButton(
                onPressed: () => stopAudio(),
                child: Text('Stop music'),
              ),
            ),
            Center(
              child: TextButton(
                onPressed: () {
                  final position = getPosition();
                  trySeek(position: position + 10);
                },
                child: Text('Skip forward 10 seconds'),
              ),
            ),
          ],
        ),
      ),
    );
  }
}
