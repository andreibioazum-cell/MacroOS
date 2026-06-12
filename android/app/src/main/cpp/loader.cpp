#include <jni.h>
#include <sys/mman.h>
#include <cstring>
extern "C" JNIEXPORT void JNICALL
Java_com_aetheria_loader_MainActivity_startKernel(JNIEnv* env, jobject, jbyteArray kernelData, jobject fb, jobject kb, jint w, jint h) {
    jsize len = env->GetArrayLength(kernelData);
    jbyte* bytes = env->GetByteArrayElements(kernelData, nullptr);
    void* fb_ptr = env->GetDirectBufferAddress(fb);
    void* kb_ptr = env->GetDirectBufferAddress(kb);
    void* exec_mem = mmap(nullptr, len, PROT_READ | PROT_WRITE | PROT_EXEC, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    memcpy(exec_mem, bytes, len);
    __builtin___clear_cache((char*)exec_mem, (char*)exec_mem + len);
    ((void (*)(void*, void*, int, int))exec_mem)(fb_ptr, kb_ptr, w, h);
}
