package com.aetheria.loader;
import android.app.Activity;
import android.graphics.*;
import android.os.Bundle;
import android.view.*;
import java.nio.ByteBuffer;

public class MainActivity extends Activity {
    static { System.loadLibrary("aetheria_loader"); }
    private native void startKernel(byte[] kernelData, ByteBuffer frameBuffer, ByteBuffer kbBuffer, int width, int height);
    private ByteBuffer frameBuffer, kbBuffer; 
    private Bitmap bitmap;
    
    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_main);
        SurfaceView surface = findViewById(R.id.surface);
        frameBuffer = ByteBuffer.allocateDirect(1280 * 720 * 4);
        kbBuffer = ByteBuffer.allocateDirect(1);
        bitmap = Bitmap.createBitmap(1280, 720, Bitmap.Config.ARGB_8888);
        
        View.OnTouchListener dpad = (v, e) -> {
            if (e.getAction() == MotionEvent.ACTION_DOWN) {
                if(v.getId() == R.id.btnW) kbBuffer.put(0, (byte)'w');
                if(v.getId() == R.id.btnS) kbBuffer.put(0, (byte)'s');
                if(v.getId() == R.id.btnA) kbBuffer.put(0, (byte)'a');
                if(v.getId() == R.id.btnD) kbBuffer.put(0, (byte)'d');
            }
            return false;
        };
        findViewById(R.id.btnW).setOnTouchListener(dpad);
        findViewById(R.id.btnS).setOnTouchListener(dpad);
        findViewById(R.id.btnA).setOnTouchListener(dpad);
        findViewById(R.id.btnD).setOnTouchListener(dpad);

        surface.getHolder().addCallback(new SurfaceHolder.Callback() {
            @Override
            public void surfaceCreated(SurfaceHolder h) {
                new Thread(() -> {
                    try {
                        byte[] kernel = getResources().openRawResource(R.raw.kernel_bin).readAllBytes();
                        startKernel(kernel, frameBuffer, kbBuffer, 1280, 720);
                    } catch(Exception e) {}
                }).start();
                new Thread(() -> {
                    while(true) {
                        Canvas c = h.lockCanvas();
                        if (c != null) {
                            frameBuffer.position(0);
                            bitmap.copyPixelsFromBuffer(frameBuffer);
                            c.drawBitmap(bitmap, null, new Rect(0,0, surface.getWidth(), surface.getHeight()), null);
                            h.unlockCanvasAndPost(c);
                        }
                        try { Thread.sleep(16); } catch(Exception e){}
                    }
                }).start();
            }
            @Override public void surfaceChanged(SurfaceHolder h, int f, int w, int h2) {}
            @Override public void surfaceDestroyed(SurfaceHolder h) {}
        });
    }
}
