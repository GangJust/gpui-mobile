package dev.gpui.mobile;

import android.app.Activity;
import android.os.Bundle;

import java.util.concurrent.CountDownLatch;
import java.util.concurrent.atomic.AtomicIntegerArray;

/**
 * Transparent helper Activity for handling runtime permission requests.
 *
 * <p>NativeActivity does not receive onRequestPermissionsResult callbacks,
 * so this lightweight Activity is used as a proxy.</p>
 */
public class GpuiPermissionActivity extends Activity {

    private static final int PERMISSION_REQUEST_CODE = 9002;

    /** Latch that the calling thread waits on. */
    static CountDownLatch sLatch;
    /** Permissions to request. */
    static String[] sPermissions;
    /** Grant results (PackageManager.PERMISSION_GRANTED or DENIED). */
    static AtomicIntegerArray sResults;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);

        if (sPermissions != null && sPermissions.length > 0) {
            requestPermissions(sPermissions, PERMISSION_REQUEST_CODE);
        } else {
            if (sLatch != null) sLatch.countDown();
            finish();
        }
    }

    @Override
    public void onRequestPermissionsResult(int requestCode, String[] permissions, int[] grantResults) {
        super.onRequestPermissionsResult(requestCode, permissions, grantResults);

        if (requestCode == PERMISSION_REQUEST_CODE && sResults != null) {
            for (int i = 0; i < grantResults.length && i < sResults.length(); i++) {
                sResults.set(i, grantResults[i]);
            }
        }

        if (sLatch != null) sLatch.countDown();
        finish();
    }

    @Override
    public void onBackPressed() {
        if (sLatch != null) sLatch.countDown();
        super.onBackPressed();
    }
}
