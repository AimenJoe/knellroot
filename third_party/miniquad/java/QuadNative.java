package quad_native;

import android.view.Surface;

// Java force the MainActivity class to belong to a specially named package
// this package name is forced to be different for each app
// and Java do not have any way to specify a native symbol lookup name..
// To workaround this - all native callbakcks lives in QuadNative class
public class QuadNative {
    // belongs to MainActivity class
    public native static void activityOnCreate(Object activity);
    public native static void activityOnResume();
    public native static void activityOnPause();
    public native static void activityOnDestroy();

    // belongs to QuadSurface class
    public native static void surfaceOnSurfaceCreated(Surface surface);
    public native static void surfaceOnSurfaceDestroyed(Surface surface);
    public native static void surfaceOnTouch(int id, int phase, float x, float y);
    public native static void surfaceOnSurfaceChanged(Surface surface, int width, int height);
    // nail-under-the-roots 패치(0171): repeat = 누르고 있어서 생긴 반복 이벤트
    public native static void surfaceOnKeyDown(int keycode, boolean repeat);
    public native static void surfaceOnKeyUp(int keycode);
    public native static void surfaceOnCharacter(int character);
    // nail-under-the-roots 패치(0143): 패드 축 여섯(왼쪽 X·Y, 오른쪽 Z·RZ, HAT X·Y)
    public native static void surfaceOnJoystick(float x, float y, float z, float rz, float hatX, float hatY);
}
