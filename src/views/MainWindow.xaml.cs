using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using System;

namespace ClearTool_WinUI
{
    public sealed partial class MainWindow : Window
    {
        public MainWindow()
        {
            this.InitializeComponent();
            //ContentFrame.Navigate(typeof(HomePage));
        }

        private void NavView_SelectionChanged(NavigationView sender, NavigationViewSelectionChangedEventArgs args)
        {
            if (args.IsSettingsSelected)
            {
                ContentFrame.Navigate(typeof(SettingsPage));
                return;
            }

            var selectedItem = args.SelectedItem as NavigationViewItem;
            if (selectedItem != null)
            {
                string pageTag = selectedItem.Tag.ToString();
                Type pageType = null;

                switch (pageTag)
                {
                    case "HomePage":
                        pageType = typeof(HomePage);
                        break;
                    case "SearchPage":
                        pageType = typeof(SearchPage);
                        break;
                    case "ClearCachePage":
                        pageType = typeof(HomePage); // Redirection temporaire vers HomePage
                        break;
                }

                if (pageType != null)
                {
                    ContentFrame.Navigate(pageType);
                }
            }
        }
    }


        // private void clearButton_Click(object sender, RoutedEventArgs e)
        // {
        //     resultTextBlock.Text = "";
        // }

        // private async void openFolderButton_Click(object sender, RoutedEventArgs e)
        // {
        //     var folderPicker = new FolderPicker
        //     {
        //         SuggestedStartLocation = PickerLocationId.Desktop,
        //         CommitButtonText = "Select Folder"
        //     };

        //     folderPicker.FileTypeFilter.Add("*");

        //     var folder = await folderPicker.PickSingleFolderAsync();
        //     if (folder != null)
        //     {
        //         string selectedPath = folder.Path;
        //         resultTextBlock.Text = selectedPath;
        //     }
        // }

        // private void TogglePaneButton_Click(object sender, RoutedEventArgs e)
        // {
        //     NavView.IsPaneOpen = !NavView.IsPaneOpen;
        // }

        // private async void searchTextBox_TextChanged(object sender, TextChangedEventArgs e)
        // {
        //     string keyword = searchTextBox.Text;

        //     if (string.IsNullOrEmpty(keyword))
        //     {
        //         resultTextBlock.Text = "";
        //         return;
        //     }

        //     try
        //     {
        //         // Appel asynchrone de la méthode SearchFiles
        //         List<string> results = await Task.Run(() => SearchFiles(keyword));

        //         // Affichage des résultats dans le TextBlock
        //         resultTextBlock.Text = results.Count > 0
        //             ? string.Join(Environment.NewLine, results)
        //             : $"No results found for: {keyword}";
        //     }
        //     catch (Exception ex)
        //     {
        //         resultTextBlock.Text = $"Error: {ex.Message}";
        //     }
        // }

        // public List<string> SearchFiles(string keyword)
        // {
        //     List<string> results = new List<string>();
        //     int resultCount = 0;
        //     IntPtr[] resultPtrs = new IntPtr[10000]; // Ajustez la taille si nécessaire

        //     try
        //     {
        //         // Appel de la fonction native pour rechercher les fichiers
        //         searchFilesInDrives(keyword, resultPtrs, ref resultCount);

        //         // Convertir chaque pointeur en chaîne de caractères
        //         for (int i = 0; i < resultCount; i++)
        //         {
        //             if (resultPtrs[i] != IntPtr.Zero)
        //             {
        //                 string filePath = Marshal.PtrToStringAnsi(resultPtrs[i]);
        //                 results.Add(filePath);
        //             }
        //         }
        //     }
        //     finally
        //     {
        //         // Libérer la mémoire native
        //         freeResults(resultPtrs, resultCount);
        //     }

        //     return results;
        // }

}
