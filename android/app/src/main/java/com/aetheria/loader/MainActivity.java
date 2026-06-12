package com.aetheria.loader;

import android.app.Activity;
import android.content.Context;
import android.graphics.*;
import android.os.Bundle;
import android.view.*;
import android.view.inputmethod.InputMethodManager;
import java.nio.ByteBuffer;

public class MainActivity extends Activity {
    static { System.loadLibrary("aetheria_loader"); }
    private native void startKernel(byte[] kernelData, ByteBuffer frameBuffer, ByteBuffer kbBuffer, int width, int height);
    
    private ByteBuffer frameBuffer, kbBuffer; 
    private Bitmap bitmap;
    private SurfaceView surface;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        
        // Используем Layout с кнопками
        setContentView(R.layout.activity_main);
        surface = findViewById(R.id.surface);

        frameBuffer = ByteBuffer.allocateDirect(1280 * 720 * 4);
        kbBuffer = ByteBuffer.allocateDirect(1);
        bitmap = Bitmap.createBitmap(1280, 720, Bitmap.Config.ARGB_8888);
        
        // --- ВКЛЮЧАЕМ КЛАВИАТУРУ ---
        surface.setFocusable(true);
        surface.setFocusableInTouchMode(true);
        
        // При нажатии на экран - показать клавиатуру
        surface.setOnTouchListener((v, event) -> {
            surface.requestFocus();
            InputMethodManager imm = (InputMethodManager) getSystemService(Context.INPUT_METHOD_SERVICE);
            imm.showSoftInput(surface, InputMethodManager.SHOW_IMPLICIT);
            return true;
        });

        // Настройка экранных кнопок (D-Pad)
        View.OnTouchListener dpadListener = (v, event) -> {
            if (event.getAction() == MotionEvent.ACTION_DOWN) {
                if(v.getId() == R.id.btnW) kbBuffer.put(0, (byte)'w');
                if(v.getId() == R.id.btnS) kbBuffer.put(0, (byte)'s');
                if(v.getId() == R.id.btnA) kbBuffer.put(0, (byte)'a');
                if(v.getId() == R.id.btnD) kbBuffer.put(0, (byte)'d');
                if(v.getId() == R.id.btnQ) kbBuffer.put(0, (byte)'q');
            }
            return false;
        };
        
        findViewById(R.id.btnW).setOnTouchListener(dpadListener);
        findViewById(R.id.btnS).setOnTouchListener(dpadListener);
        findViewById(R.id.btnA).setOnTouchListener(dpadListener);
        findViewById(R.id.btnD).setOnTouchListener(dpadListener);
        findViewById(R.id.btnQ).setOnTouchListener(dpadListener);

        // Слушатель физической или экранной клавиатуры
        surface.setOnKeyListener((v, keyCode, event) -> {
            if (event.getAction() == KeyEvent.ACTION_DOWN) {
                int c = event.getUnicodeChar();
                if (keyCode == KeyEvent.KEYCODE_DEL) c = 8;     // Backspace
                if (keyCode == KeyEvent.KEYCODE_ENTER) c = 10;  // Enter
                if (c != 0) kbBuffer.put(0, (byte)c);
                return true;
            }
            return false;
        });

        // Запуск отрисовки
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
