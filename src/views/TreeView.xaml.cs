using System.Collections.Generic;
using System.IO;
using Microsoft.UI.Xaml.Controls;

public class DirectoryNode
{
    public string Name { get; set; }
    public string Path { get; set; }
    public List<DirectoryNode> Children { get; set; }

    public DirectoryNode(string name, string path)
    {
        Name = name;
        Path = path;
        Children = new List<DirectoryNode>();
    }
}

public class DirectoryHelper
{
    public static DirectoryNode GetDirectories(string path)
    {
        var directoryInfo = new DirectoryInfo(path);
        var node = new DirectoryNode(directoryInfo.Name, directoryInfo.FullName);

        try
        {
            foreach (var dir in directoryInfo.GetDirectories())
            {
                node.Children.Add(GetDirectories(dir.FullName));
            }
        }
        catch { /* Gestion des exceptions pour les dossiers non accessibles */ }

        return node;
    }
}
