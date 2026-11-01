package com.alux.browser

import android.os.Bundle
import android.view.View
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsCompat

class MainActivity : TauriActivity() {
  // Alux patch: three fixes to Tauri's generated activity.
  //
  // 1. TauriActivity sets `handleBackNavigation = false`, which disables
  //    WryActivity's OnBackPressedCallback — the Android back key then does
  //    nothing inside a page. Re-enable it: it calls WebView.goBack() when
  //    there is history and finishes the activity otherwise.
  //
  // 2. Android 15 (API 35+) forces edge-to-edge drawing for apps that target
  //    SDK 35+, so removing enableEdgeToEdge() is not enough: page headers and
  //    in-page toolbars still sit under the clock/battery. Pad the content view
  //    by the system-bar insets instead.
  //
  // 3. Tauri's template calls enableEdgeToEdge() in onCreate — Alux does not.
  override val handleBackNavigation: Boolean = true

  override fun onCreate(savedInstanceState: Bundle?) {
    super.onCreate(savedInstanceState)

    val root = findViewById<View>(android.R.id.content)
    ViewCompat.setOnApplyWindowInsetsListener(root) { view, insets ->
      val bars = insets.getInsets(WindowInsetsCompat.Type.systemBars())
      view.setPadding(bars.left, bars.top, bars.right, bars.bottom)
      WindowInsetsCompat.CONSUMED
    }
  }
}
