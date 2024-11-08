using Microsoft.UI.Xaml.Controls;
using System;
using System.Collections.ObjectModel;
using ClearTool_WinUI.Models;

namespace ClearTool_WinUI
{
    public sealed partial class SearchPage : Page
    {
        private readonly ObservableCollection<SearchResultItem> _mockResults;

        public SearchPage()
        {
            _mockResults = new ObservableCollection<SearchResultItem>();
            this.InitializeComponent();
            InitializeMockData();
            InitializeControls();
        }

        private void InitializeMockData()
        {
            _mockResults.Clear();
            _mockResults.Add(new SearchResultItem
            {
                Path = "C:\\Users\\Documents\\rapport.docx",
                Size = 1024576,
                LastModified = DateTime.Now.AddDays(-1)
            });
            _mockResults.Add(new SearchResultItem
            {
                Path = "C:\\Users\\Images\\photo.jpg",
                Size = 2048576,
                LastModified = DateTime.Now.AddDays(-2)
            });
            _mockResults.Add(new SearchResultItem
            {
                Path = "C:\\Users\\Downloads\\setup.exe",
                Size = 5242880,
                LastModified = DateTime.Now.AddHours(-12)
            });

            ResultsList.ItemsSource = _mockResults;
        }

        private void InitializeControls()
        {
            DriveSelector.ItemsSource = new[] { "C: (Windows)", "D: (Data)", "E: (Backup)" };
            DriveSelector.SelectedIndex = 0;
        }
    }
}