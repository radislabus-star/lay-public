pub(crate) fn component_xml(exec_path: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<component>
  <name>org.freedesktop.IBus.Lay</name>
  <description>Lay input-method bridge</description>
  <exec>{exec_path} --ibus --managed</exec>
  <version>{}</version>
  <author>radislabus</author>
  <license>MIT</license>
  <homepage>https://github.com/radislabus-star/lay-public</homepage>
  <textdomain>lay</textdomain>
  <engines>
    <engine>
      <name>lay-ime-us</name>
      <language>en</language>
      <license>MIT</license>
      <author>radislabus</author>
      <icon>input-keyboard</icon>
      <layout>us</layout>
      <longname>Lay IME US</longname>
      <description>Lay IME US input-method bridge</description>
      <rank>50</rank>
    </engine>
    <engine>
      <name>lay-ime-ru</name>
      <language>ru</language>
      <license>MIT</license>
      <author>radislabus</author>
      <icon>input-keyboard</icon>
      <layout>ru</layout>
      <longname>Lay IME RU</longname>
      <description>Lay IME RU input-method bridge</description>
      <rank>50</rank>
    </engine>
  </engines>
</component>
"#,
        env!("CARGO_PKG_VERSION")
    )
}
