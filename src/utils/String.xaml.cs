using Microsoft.UI.Xaml.Controls;

namespace ClearTool_WinUI.Utils
{
    public static class TextUtils
    {
        public static void AppendTextField(TextBlock textBlock, string text)
        {
            textBlock.Text += text + "\n";
        }
    }
}
