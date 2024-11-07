using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Threading.Tasks;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Windows.Storage.Pickers;

namespace ClearTool_WinUI
{
    public sealed partial class MainWindow : Window
    {

        [DllImport("SearchFiles.dll", CharSet = CharSet.Ansi, CallingConvention = CallingConvention.Cdecl)]
        public static extern void searchFilesInDrives(string keyword, [Out] IntPtr[] results, ref int resultCount);

        [DllImport("SearchFiles.dll", CharSet = CharSet.Ansi, CallingConvention = CallingConvention.Cdecl)]
        public static extern void freeResults(IntPtr[] results, int resultCount);

        private readonly List<string> selectedPaths = new List<string>();

        public MainWindow()
        {
            try
            {
                this.InitializeComponent();
                this.Title = "ClearTool \u00A9 SUP2Ak"; ;
            }
            catch (Exception ex)
            {
                Console.WriteLine($"Une erreur s'est produite : {ex.Message}");
                Console.WriteLine(ex.StackTrace);
                Console.ReadLine();
            }
        }

        private void clearButton_Click(object sender, RoutedEventArgs e)
        {
            resultTextBlock.Text = "";
        }

        private async void openFolderButton_Click(object sender, RoutedEventArgs e)
        {
            var folderPicker = new FolderPicker
            {
                SuggestedStartLocation = PickerLocationId.Desktop,
                CommitButtonText = "Select Folder"
            };

            folderPicker.FileTypeFilter.Add("*");

            var folder = await folderPicker.PickSingleFolderAsync();
            if (folder != null)
            {
                string selectedPath = folder.Path;
                resultTextBlock.Text = selectedPath;
            }
        }

        private async void searchTextBox_TextChanged(object sender, TextChangedEventArgs e)
        {
            string keyword = searchTextBox.Text;

            if (string.IsNullOrEmpty(keyword))
            {
                resultTextBlock.Text = "";
                return;
            }

            try
            {
                // Appel asynchrone de la méthode SearchFiles
                List<string> results = await Task.Run(() => SearchFiles(keyword));

                // Affichage des résultats dans le TextBlock
                resultTextBlock.Text = results.Count > 0
                    ? string.Join(Environment.NewLine, results)
                    : $"No results found for: {keyword}";
            }
            catch (Exception ex)
            {
                resultTextBlock.Text = $"Error: {ex.Message}";
            }
        }

        public List<string> SearchFiles(string keyword)
        {
            List<string> results = new List<string>();
            int resultCount = 0;
            IntPtr[] resultPtrs = new IntPtr[10000]; // Ajustez la taille si nécessaire

            try
            {
                // Appel de la fonction native pour rechercher les fichiers
                searchFilesInDrives(keyword, resultPtrs, ref resultCount);

                // Convertir chaque pointeur en chaîne de caractères
                for (int i = 0; i < resultCount; i++)
                {
                    if (resultPtrs[i] != IntPtr.Zero)
                    {
                        string filePath = Marshal.PtrToStringAnsi(resultPtrs[i]);
                        results.Add(filePath);
                    }
                }
            }
            finally
            {
                // Libérer la mémoire native
                freeResults(resultPtrs, resultCount);
            }

            return results;
        }

    }
}
