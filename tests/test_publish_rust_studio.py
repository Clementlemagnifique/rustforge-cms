"""Safety checks for generated asset publication; all writes use temporary directories."""
import importlib.util
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location("publisher", Path(__file__).resolve().parents[1] / "scripts/publish-rust-studio.py")
publisher = importlib.util.module_from_spec(spec)
spec.loader.exec_module(publisher)


class PublisherTests(unittest.TestCase):
    def source(self, root, version):
        source = root / ("build-" + version)
        source.mkdir()
        (source / "index.html").write_text('<script type="module" src="/admin/rust/bootstrap-' + version + '.js"></script>')
        (source / ("bootstrap-" + version + ".js")).write_text("import '/admin/rust/app-" + version + ".js'; const wasm='/admin/rust/app-" + version + ".wasm';")
        (source / ("app-" + version + ".js")).write_text("export const version='" + version + "';")
        (source / ("app-" + version + ".wasm")).write_bytes(b"\0asm\x01\0\0\0")
        (source / ".trunk-lock").touch()
        return source

    def test_publication_archive_restore_and_idempotence(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            destinations = {"preview": root / "preview", "served": root / "served"}
            first = self.source(root, "first")
            second = self.source(root, "second")
            archives = root / "archives"
            publisher.publish(first, destinations, archives)
            before = {key: publisher.inventory(path) for key, path in destinations.items()}
            result = publisher.publish(second, destinations, archives)
            self.assertEqual(result["archived_files"], 8)
            for destination in destinations.values():
                self.assertEqual(publisher.inventory(destination), publisher.validate(second))
                self.assertFalse((destination / ".trunk-lock").exists())
            self.assertEqual(publisher.publish(second, destinations, archives)["archived_files"], 0)
            publisher.restore(Path(result["archive"]), destinations)
            for key, destination in destinations.items():
                for name, expected in before[key].items():
                    self.assertEqual(publisher.digest(destination / name), expected)

    def test_missing_dependency_cannot_change_existing_output(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            good = self.source(root, "good")
            destination = root / "served"
            publisher.publish(good, {"served": destination}, root / "archives")
            before = publisher.inventory(destination)
            bad = self.source(root, "bad")
            (bad / "app-bad.wasm").unlink()
            with self.assertRaises(ValueError):
                publisher.publish(bad, {"served": destination}, root / "archives")
            self.assertEqual(publisher.inventory(destination), before)

    def test_symlink_is_not_followed(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = self.source(root, "safe")
            (source / "outside.js").symlink_to(root / "outside")
            with self.assertRaises(ValueError):
                publisher.publish(source, {"served": root / "served"}, root / "archives")

    def test_restore_refuses_a_later_unrelated_index(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            dest = {"served": root / "served"}
            publisher.publish(self.source(root, "one"), dest, root / "archives")
            result = publisher.publish(self.source(root, "two"), dest, root / "archives")
            (dest["served"] / "index.html").write_text("another build")
            with self.assertRaises(ValueError):
                publisher.restore(Path(result["archive"]), dest)
            self.assertEqual((dest["served"] / "index.html").read_text(), "another build")


if __name__ == "__main__":
    unittest.main()
