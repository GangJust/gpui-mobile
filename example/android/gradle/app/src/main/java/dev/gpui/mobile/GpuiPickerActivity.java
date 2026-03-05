package dev.gpui.mobile;

import android.app.Activity;
import android.content.Intent;
import android.net.Uri;
import android.os.Bundle;

import java.util.ArrayList;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.atomic.AtomicReference;

/**
 * Transparent helper Activity that handles startActivityForResult calls.
 *
 * <p>NativeActivity cannot easily receive activity results, so this lightweight
 * transparent Activity is used as a proxy. It launches the requested intent,
 * captures the result, stores it in a static field, and finishes itself.</p>
 *
 * <p>Called from Rust via JNI through GpuiFilePicker / GpuiImagePicker.</p>
 */
public class GpuiPickerActivity extends Activity {

    private static final int REQUEST_CODE = 9001;

    /** Latch that the calling thread waits on. */
    static CountDownLatch sLatch;
    /** Result URIs from the picker. Null means cancelled. */
    static AtomicReference<ArrayList<String>> sResult = new AtomicReference<>(null);
    /** The intent to launch. Set before starting this Activity. */
    static Intent sPendingIntent;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);

        if (sPendingIntent != null) {
            try {
                startActivityForResult(sPendingIntent, REQUEST_CODE);
            } catch (Exception e) {
                android.util.Log.e("GpuiPicker", "Failed to start picker intent", e);
                sResult.set(null);
                if (sLatch != null) sLatch.countDown();
                finish();
            }
        } else {
            sResult.set(null);
            if (sLatch != null) sLatch.countDown();
            finish();
        }
    }

    @Override
    protected void onActivityResult(int requestCode, int resultCode, Intent data) {
        super.onActivityResult(requestCode, resultCode, data);

        if (requestCode == REQUEST_CODE) {
            if (resultCode == RESULT_OK && data != null) {
                ArrayList<String> uris = new ArrayList<>();

                // Check for multiple results (e.g. multi-select)
                if (data.getClipData() != null) {
                    int count = data.getClipData().getItemCount();
                    for (int i = 0; i < count; i++) {
                        Uri uri = data.getClipData().getItemAt(i).getUri();
                        if (uri != null) {
                            uris.add(uri.toString());
                        }
                    }
                } else if (data.getData() != null) {
                    uris.add(data.getData().toString());
                }

                sResult.set(uris);
            } else {
                sResult.set(null); // cancelled
            }
        } else {
            sResult.set(null);
        }

        if (sLatch != null) sLatch.countDown();
        finish();
    }

    @Override
    public void onBackPressed() {
        sResult.set(null);
        if (sLatch != null) sLatch.countDown();
        super.onBackPressed();
    }
}
