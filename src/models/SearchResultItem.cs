using System;

namespace ClearTool_WinUI.Models
{
    public class SearchResultItem
    {
        public string Path { get; set; }
        public long Size { get; set; }
        public DateTime LastModified { get; set; }

        public string SizeFormatted
        {
            get
            {
                if (Size < 1024) return $"{Size} B";
                if (Size < 1048576) return $"{Size / 1024:F1} KB";
                if (Size < 1073741824) return $"{Size / 1048576:F1} MB";
                return $"{Size / 1073741824:F1} GB";
            }
        }
    }
}