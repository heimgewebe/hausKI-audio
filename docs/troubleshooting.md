# Troubleshooting

Dieses Dokument hilft bei der Diagnose und Behebung häufiger Probleme mit der
Audio-Verarbeitung.

## Knackser und Aussetzer

Audio-Knackser ("clicks") oder kurze Aussetzer ("dropouts") sind oft ein
Zeichen dafür, dass das System die Audio-Daten nicht schnell genug
verarbeiten kann.

### Mögliche Ursachen und Lösungen

- **Systemlast**: Überprüfen Sie die CPU-Auslastung. Andere Prozesse könnten
  dem Audio-System die Ressourcen streitig machen. Schließen Sie nicht
  benötigte Anwendungen.
- **Buffer Size**: Eine zu kleine Puffergröße (Buffer Size) bei der
  Audio-Schnittstelle kann zu Aussetzern führen. Erhöhen Sie den Wert
  schrittweise in Ihrer ALSA- oder JACK-Konfiguration.
- **Sample Rate**: Stellen Sie sicher, dass alle Teile Ihrer Audio-Kette
  (Aufnahme, Plugins, Wiedergabe) mit derselben Abtastrate (Sample Rate)
  arbeiten, z.B. 48000 Hz.

## Latenz

Latenz ist die Zeitverzögerung zwischen dem Eingang eines Audio-Signals und
dem Zeitpunkt, zu dem es wieder ausgegeben wird.

### Latenz reduzieren

- **Puffergröße**: Eine kleinere Puffergröße reduziert die Latenz, erhöht
  aber das Risiko von Knacksern. Finden Sie hier einen Kompromiss.
- **Real-Time Kernel**: Für anspruchsvolle Audio-Anwendungen unter Linux wird
  ein Real-Time-Kernel empfohlen. Dieser priorisiert Audio-Threads und sorgt
  für eine stabilere Verarbeitung.

## ALSA-Konfiguration prüfen

`ALSA (Advanced Linux Sound Architecture)` ist die Grundlage für Audio unter
Linux. Falsche Konfigurationen können viele Probleme verursachen.

- **Gerät auswählen**: Stellen Sie sicher, dass das richtige Aufnahme- und
  Wiedergabegerät in Ihrer Anwendung ausgewählt ist. Mit dem Befehl `aplay -l`
  und `arecord -l` können Sie die verfügbaren Geräte auflisten.
- **Mixer-Einstellungen**: Verwenden Sie `alsamixer` im Terminal, um die
  Lautstärkepegel und eventuelle Stummschaltungen (Mute) zu überprüfen. Ein
  Kanal könnte versehentlich stummgeschaltet sein.
- **Rechte**: Ihr Benutzer muss Mitglied der `audio`-Gruppe sein, um direkten
  Zugriff auf die Audio-Hardware zu haben.
