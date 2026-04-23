import React, { useRef, useCallback, useEffect } from 'react';
import {
  View,
  Text,
  Modal,
  TouchableOpacity,
  StyleSheet,
  Platform,
  ActivityIndicator,
} from 'react-native';
import { CameraView, useCameraPermissions } from 'expo-camera';
import { theme } from '../lib/theme';

const mono = Platform.OS === 'ios' ? 'Menlo' : 'monospace';

type Props = {
  visible: boolean;
  onClose: () => void;
  onPairingScanned: (data: string) => void;
};

export default function PairingQrScanner({ visible, onClose, onPairingScanned }: Props) {
  const [permission, requestPermission] = useCameraPermissions();
  const lockedRef = useRef(false);

  useEffect(() => {
    if (!visible) lockedRef.current = false;
  }, [visible]);

  const handleBarcode = useCallback(
    ({ data }: { data: string }) => {
      if (lockedRef.current) return;
      const t = data.trim();
      if (!t) return;
      lockedRef.current = true;
      onPairingScanned(t);
      onClose();
    },
    [onPairingScanned, onClose]
  );

  const openPermission = useCallback(() => {
    void requestPermission();
  }, [requestPermission]);

  return (
    <Modal visible={visible} animationType="slide" onRequestClose={onClose}>
      <View style={styles.container}>
        <View style={styles.toolbar}>
          <TouchableOpacity onPress={onClose} style={styles.toolbarBtn}>
            <Text style={[styles.toolbarText, { color: '#ff8888' }]}>[ cancel ]</Text>
          </TouchableOpacity>
          <Text style={styles.title}>scan pairing qr</Text>
          <View style={{ width: 72 }} />
        </View>

        {!permission?.granted ? (
          <View style={styles.centered}>
            <Text style={styles.help}>Camera access is required to scan a pairing QR code.</Text>
            <TouchableOpacity onPress={openPermission} style={styles.permBtn}>
              <Text style={styles.permBtnText}>[ grant camera ]</Text>
            </TouchableOpacity>
            {permission?.canAskAgain === false ? (
              <Text style={styles.help}>Enable camera in system settings for this app.</Text>
            ) : null}
          </View>
        ) : (
          <CameraView
            style={styles.camera}
            facing="back"
            barcodeScannerSettings={{ barcodeTypes: ['qr'] }}
            onBarcodeScanned={handleBarcode}
          />
        )}

        {permission?.granted ? (
          <View style={styles.footer}>
            <ActivityIndicator color={theme.colors.primarySolid} />
            <Text style={styles.footerText}>Point at the QR from desktop Jarvis relay pairing.</Text>
          </View>
        ) : null}
      </View>
    </Modal>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: theme.colors.bg },
  toolbar: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    paddingTop: 48,
    paddingHorizontal: 12,
    paddingBottom: 12,
    borderBottomWidth: 1,
    borderBottomColor: theme.colors.border,
  },
  toolbarBtn: { padding: 8 },
  toolbarText: { fontFamily: mono, fontSize: 11 },
  title: { fontFamily: mono, fontSize: 11, color: theme.colors.primary },
  centered: { flex: 1, justifyContent: 'center', padding: 24 },
  help: { fontFamily: mono, fontSize: 11, color: theme.colors.text, textAlign: 'center', marginBottom: 16 },
  permBtn: {
    alignSelf: 'center',
    borderWidth: 1,
    borderColor: theme.colors.border,
    paddingVertical: 10,
    paddingHorizontal: 16,
    borderRadius: 4,
  },
  permBtnText: { fontFamily: mono, fontSize: 12, color: theme.colors.primary },
  camera: { flex: 1 },
  footer: {
    padding: 16,
    alignItems: 'center',
    gap: 8,
    borderTopWidth: 1,
    borderTopColor: theme.colors.border,
  },
  footerText: { fontFamily: mono, fontSize: 10, color: theme.colors.tabInactive, textAlign: 'center' },
});
