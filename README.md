# CPU Path Tracing

一个使用 Rust 从零实现的 CPU 路径追踪渲染器。

## 项目简介 Intro
本项目是 [CPUPathTracing](https://github.com/HeaoYe/CPUPathTracing) 的 Rust 实现。原项目是一个从零实现、面向教学的 C++17 CPU 基于物理的 路径追踪渲染器，同时也是 [Bilibili 公开课](https://www.bilibili.com/video/BV1MJYAeYEDk) 的配套代码仓库。

`CPUPathTracing-rs` 将会按照课程进度 1:1 实现每一讲最终应具备的功能与效果，每一讲的代码都会对应一个branch。

两套实现的算法目标与课程进度保持对应，但具体的数据组织、抽象方式等可能不同。

## 项目状态 Status
当前进度：**3 / 25 — 自旋锁与并行 for 循环**

- [x] [Lecture01](../../tree/Lecture01) 课程介绍
- [x] [Lecture02](../../tree/Lecture02) 线程池与胶片
- [x] [Lecture03](../../tree/Lecture03) 自旋锁与并行 for 循环
- [ ] Lecture04 球体与相交测试
- [ ] Lecture05 模型渲染
- [ ] Lecture06 平面与场景
- [ ] Lecture07 材质与极简光追
- [ ] Lecture08 一些代码重构
- [ ] Lecture09 性能优化（上）：并行优化
- [ ] Lecture10 性能优化（中）：高性能 BVH 加速结构
- [ ] Lecture11 性能优化（下）：场景管理
- [ ] Lecture12 路径追踪与重要性采样
- [ ] Lecture13 代码勘误与新材质类
- [ ] Lecture14 电介质与导体
- [ ] Lecture15 往期勘误与代码重构
- [ ] Lecture16 微表面理论
- [ ] Lecture17 实时预览
- [ ] Lecture18 向光源采样
- [ ] Lecture19 多重重要性采样
- [ ] Lecture20 BVH 构建优化
- [ ] Lecture21 环境光照
- [ ] Lecture22 代码勘误和一些改进
- [ ] Lecture23 光谱渲染（上）色彩科学
- [ ] Lecture24-1 光谱渲染（中）基础框架
- [ ] Lecture24-2 光谱渲染（中）Spectral MIS
- [ ] Lecture25 光谱渲染（下）RGB 转光谱

## 代码构建 Build
### 获取源码
```bash
git clone https://github.com/HeaoYe/CPUPathTracing-rs.git
```

### 编译
```bash
cargo build --release
```

### 运行
```bash
cargo run --release
```

## 参考资料 References
- [Physically Based Rendering: From Theory to Implementation, 4th Edition](https://pbr-book.org/4ed/contents)
  Matt Pharr, Wenzel Jakob, Greg Humphreys, 2023.

- [Robust Monte Carlo Methods for Light Transport Simulation](https://graphics.stanford.edu/papers/veach_thesis/)
  Eric Veach, Ph.D. dissertation, Stanford University, 1997.

- [On fast Construction of SAH-based Bounding Volume Hierarchies](https://publications.sci.utah.edu/publications/wald07/fastbuild.pdf)
  Ingo Wald, IEEE/Eurographics Symposium on Interactive Ray Tracing, 33–40, 2007.

- [Microfacet Models for Refraction through Rough Surfaces](https://www.cs.cornell.edu/~srm/publications/EGSR07-btdf.html)
  Bruce Walter, Stephen R. Marschner, Hongsong Li, Kenneth E. Torrance, Eurographics Symposium on Rendering (EGSR), 195–206, 2007.

- [Understanding the Masking-Shadowing Function in Microfacet-Based BRDFs](https://jcgt.org/published/0003/02/03/)
  Eric Heitz, Journal of Computer Graphics Techniques (JCGT), 3(2), 48–107, 2014.

- [Sampling the GGX Distribution of Visible Normals](https://jcgt.org/published/0007/04/01/)
  Eric Heitz, Journal of Computer Graphics Techniques (JCGT), 7(4), 1–13, 2018.

- [A re-determination of the trichromatic coefficients of the spectral colours](https://doi.org/10.1088/1475-4878/30/4/301)
  W. D. Wright, Transactions of the Optical Society, 30(4), 141–164, 1929.

- [A re-determination of the mixture curves of the spectrum](https://doi.org/10.1088/1475-4878/31/4/303)
  W. D. Wright, Transactions of the Optical Society, 31(4), 201–218, 1930.

- [The Colorimetric Properties of the Spectrum](https://doi.org/10.1098/rsta.1932.0005)
  John Guild, Philosophical Transactions of the Royal Society A, 230, 149–187, 1932.

- [How the CIE 1931 color-matching functions were derived from Wright-Guild data](https://doi.org/10.1002/%28SICI%291520-6378%28199702%2922%3A1%3C11%3A%3AAID-COL4%3E3.0.CO%3B2-7)
  Hugh S. Fairman, Michael H. Brill, Henry Hemmendinger, Color Research & Application, 22(1), 11–23, 1997.

- [A critical review of the development of the CIE1931 RGB color-matching functions](https://doi.org/10.1002/col.20020)
  Arthur D. Broadbent, Color Research & Application, 29(4), 267–272, 2004.

- [How the CIE 1931 RGB Color Matching Functions Were Developed from the Initial Color Matching Experiments](https://yuhaozhu.com/blog/cmf.html)
  Yuhao Zhu, blog article, 2020.

- [Hero Wavelength Spectral Sampling](https://doi.org/10.1111/cgf.12419)
  Alexander Wilkie, Sehera Nawaz, Marc Droske, Andrea Weidlich, Johannes Hanika, Computer Graphics Forum, 33(4), 123–131, 2014.

- [A Low-Dimensional Function Space for Efficient Spectral Upsampling](https://rgl.epfl.ch/publications/Jakob2019Spectral)
  Wenzel Jakob, Johannes Hanika, Computer Graphics Forum (Proceedings of Eurographics), 38(2), 147–155, 2019.

## 许可证 License
本项目原创源代码基于 [MIT License](LICENSE) 开源。
