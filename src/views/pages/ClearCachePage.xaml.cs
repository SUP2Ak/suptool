using Microsoft.UI.Xaml.Controls;
using System;
//using ClearTool_WinUI.ViewModels;

namespace ClearTool_WinUI
{
    public sealed partial class ClearCachePage : Page
    {
        // public SearchViewModel ViewModel { get; }

        public ClearCachePage()
        {
            this.InitializeComponent();
            //ViewModel = new SearchViewModel(DispatcherQueue);
            //this.DataContext = ViewModel;

            InitializeControls();
        }

        private void InitializeControls()
        {
            // Initialisation du sélecteur de lecteur
            //DriveSelector.SelectionChanged += (s, e) =>
            //{
            //    // À implémenter
            //};

            //// Initialisation du filtre d'extension
            //ExtensionFilter.TextChanged += (s, e) =>
            //{
            //    // À implémenter
            //};

            //// Initialisation des sélecteurs de date
            //DateFromPicker.SelectedDateChanged += (s, e) =>
            //{
            //    // À implémenter
            //};

            //DateToPicker.SelectedDateChanged += (s, e) =>
            //{
            //    // À implémenter
            //};

            //// Initialisation du bouton de nettoyage
            //ClearButton.Click += async (s, e) =>
            //{
            //    // À implémenter
            //};
        }

        private void LogMessage(string message)
        {
            LogTextBlock.Text += $"{DateTime.Now:HH:mm:ss} - {message}\n";
        }
    }
}

/**
src/
├── Views/
│   ├── MainWindow.xaml
│   ├── MainWindow.xaml.cs
│   ├── Pages/
│   │   ├── HomePage.xaml
│   │   ├── HomePage.xaml.cs
│   │   ├── SearchPage.xaml
│   │   ├── SearchPage.xaml.cs
│   │   ├── ClearCachePage.xaml
│   │   ├── ClearCachePage.xaml.cs
│   │   ├── SettingsPage.xaml
│   │   └── SettingsPage.xaml.cs
├── Models/
│   └── SearchResultItem.cs
└── ViewModels/
    └── SearchViewModel.cs
*/