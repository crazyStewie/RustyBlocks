use vulkano;
use glfw;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::ffi::CString;
use raylib::rgui::DrawResult::Selected;
use vulkano::VulkanObject;
use glfw::{Context};
use std::ptr::null;
use std::borrow::Borrow;
use vulkano::instance::PhysicalDevice;
use vulkano::swapchain::{SupportedPresentModes, PresentMode, Capabilities, Swapchain, CompositeAlpha, FullscreenExclusive, Surface, acquire_next_image};
use vulkano::image::{SwapchainImage, ImageUsage};
use vulkano::sync::{SharingMode, GpuFuture};
use vulkano::device::{DeviceExtensions, Device};
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::vertex::{BufferlessDefinition, BufferlessVertices};
use vulkano::framebuffer::{RenderPassAbstract, Subpass, FramebufferAbstract, Framebuffer};
use vulkano::descriptor::PipelineLayoutAbstract;
use std::ops::{Deref, DerefMut};
use vulkano::command_buffer::{AutoCommandBuffer, AutoCommandBufferBuilder, DynamicState};
use vulkano::pipeline::raster::DepthBiasControl::Dynamic;

#[cfg(all(debug_assertions))]
const ENABLE_VALIDATION_LAYERS: bool = true;
#[cfg(not(debug_assertions))]
const ENABLE_VALIDATION_LAYERS: bool = false;

const VALIDATION_LAYERS: &[&str] = &[
    "VK_LAYER_LUNARG_standard_validation"
];

//TODO, FIXME
struct Window (glfw::Window);
impl Deref for Window {
    type Target = glfw::Window;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Window {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
unsafe impl Send for Window{}
unsafe impl Sync for Window{}
pub struct RenderServer {
    glfw : glfw::Glfw,
    events: Receiver<(f64, glfw::WindowEvent)>,

    surface: Arc<vulkano::swapchain::Surface<Window>>,

    instance : Arc<vulkano::instance::Instance>,
    debug_callback: Option<vulkano::instance::debug::DebugCallback>,

    physical_device_index: usize,
    device: Arc<vulkano::device::Device>,

    graphics_queue: Arc<vulkano::device::Queue>,
    present_queue: Arc<vulkano::device::Queue>,

    swap_chain: Arc<vulkano::swapchain::Swapchain<Window>>,
    swap_chain_images: Vec<Arc<vulkano::image::SwapchainImage<Window>>>,

    render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    graphics_pipeline: Arc<GraphicsPipeline<BufferlessDefinition, Box<dyn PipelineLayoutAbstract + Send + Sync + 'static>, Arc<dyn RenderPassAbstract + Send + Sync + 'static>>>,

    swap_chain_framebuffers: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
    command_buffers: Vec<Arc<AutoCommandBuffer>>,
}

impl RenderServer {
    fn get_required_extensions(glfw : &glfw::Glfw) -> Result<vulkano::instance::InstanceExtensions, vulkano_glfw::VulkanoGlfwError> {
        let exts = glfw.get_required_instance_extensions();
        if exts.is_none() {
            return Err(vulkano_glfw::VulkanoGlfwError::NoExtensions);
        }
        let iter = exts.unwrap().into_iter().map(|s| {
            let new_c_string = CString::new(s);
            new_c_string.unwrap()
        });
        let raw_extensions = vulkano::instance::RawInstanceExtensions::new(iter);
        let mut extensions = vulkano::instance::InstanceExtensions::from(&raw_extensions);
        extensions.ext_debug_utils = true;
        Ok(extensions)
    }

    fn device_extensions() -> DeviceExtensions {
        DeviceExtensions {
            khr_swapchain: true,
            .. vulkano::device::DeviceExtensions::none()
        }
    }

    fn check_validation_layer_support() -> bool {
        let layers: Vec<_> = vulkano::instance::layers_list().unwrap().map(|l|
                l.name().to_owned())
            .collect();
        VALIDATION_LAYERS.iter().all(|layer_name|
            layers.contains(&layer_name.to_string()))
    }

    fn check_device_extension_support(device: &PhysicalDevice) -> bool {
        let avaliable_extensions = DeviceExtensions::supported_by_device(*device);
        let device_extensions = Self::device_extensions();
        return avaliable_extensions.intersection(&device_extensions) == device_extensions;
    }

    fn create_instance(glfw: &glfw::Glfw) -> Arc<vulkano::instance::Instance> {
        if ENABLE_VALIDATION_LAYERS && !Self::check_validation_layer_support() {
            println!("Validation layers requested, but not avaliable!")
        }
        let supported_extensions = vulkano::instance::InstanceExtensions::supported_by_core()
            .expect("Failed to retrieve supported extensions");
        println!("Supported extensions: {:?}", supported_extensions);

        let app_info = vulkano::instance::ApplicationInfo {
            application_name: Some("Rusty Blocks".into()),
            application_version: Some(vulkano::instance::Version { major: 0, minor: 1, patch: 0 }),
            engine_name: Some("Rusty Engine".into()),
            engine_version: Some(vulkano::instance::Version { major: 0, minor: 1, patch: 0 }),
        };

        let required_extensions = Self::get_required_extensions(glfw)
            .expect("Unable to get required extensions");
        if ENABLE_VALIDATION_LAYERS && Self::check_validation_layer_support() {
            vulkano::instance::Instance::new(Some(&app_info), &required_extensions, VALIDATION_LAYERS.iter().cloned())
                .expect("Failed to create Vulkan instance")
        }
        else {
            vulkano::instance::Instance::new(Some(&app_info), &required_extensions, None)
                .expect("Failed to create Vulkan instance")
        }
    }

    fn create_debug_callback(instance: &Arc<vulkano::instance::Instance>) -> Option<vulkano::instance::debug::DebugCallback> {
        if !ENABLE_VALIDATION_LAYERS {
            return None;
        }

        let msg_types = vulkano::instance::debug::MessageType {
            general: true,
            validation: true,
            performance: true
        };
        let msg_severities = vulkano::instance::debug::MessageSeverity {
            error: true,
            warning: true,
            information: true,
            verbose: true
        };
        vulkano::instance::debug::DebugCallback::new(instance, msg_severities, msg_types, |msg| {
            println!("validation layer {:?}", msg.description);
        }).ok()
    }

    fn create_logical_device(instance: &Arc<vulkano::instance::Instance>, device_index: usize)
        -> (Arc<vulkano::device::Device>, Arc<vulkano::device::Queue>, Arc<vulkano::device::Queue>) {
        let physical_device = vulkano::instance::PhysicalDevice::from_index(instance, device_index).unwrap();
        let queue_family = physical_device.queue_families().find(|&q| {
            q.supports_graphics()
        }).unwrap();

        let queue_priority = 1.0;

        let (device, mut queues) = vulkano::device::Device::new(physical_device, &vulkano::device::Features::none(), &Self::device_extensions(),
        [(queue_family, queue_priority)].iter().cloned())
            .expect("Failed to create logical vulkan device");

        let graphics_queue = queues.next().unwrap();
        let present_queue = queues.next().unwrap_or_else(|| graphics_queue.clone());
        return (device, graphics_queue, present_queue);
    }

    fn create_surface(instance: &Arc<vulkano::instance::Instance>, window : Window) -> Arc<vulkano::swapchain::Surface<Window>> {
        let mut internal_surface: vk_sys::SurfaceKHR = 0;
        let result = unsafe {
            glfw::ffi::glfwCreateWindowSurface(
                instance.internal_object(),
                window.window_ptr(),
                null(),
                &mut internal_surface,
            )
        };

        if result != vk_sys::SUCCESS {
            panic!("Unable to create vulkan surface");
        }
        Arc::new(unsafe{(vulkano::swapchain::Surface::from_raw_surface(instance.clone(), internal_surface, window))})
    }

    fn pick_physical_device(instance: &Arc<vulkano::instance::Instance>,surface: &Arc<Surface<Window>>) -> usize {
        vulkano::instance::PhysicalDevice::enumerate(instance)
            .position(|device| Self::is_device_suitable(surface,&device))
            .unwrap()
    }

    fn is_device_suitable(surface:&Arc<Surface<Window>>, device: &vulkano::instance::PhysicalDevice) -> bool {
        println!("Found {} queue families for device {}", device.queue_families().count(), device.name());
        let mut queue_suitable = false;
        for (id, queue_family) in device.queue_families().enumerate() {
            if queue_family.supports_graphics()  && surface.is_supported(queue_family).unwrap() {
                queue_suitable = true;
            }
        }

        let extensions_supported = Self::check_device_extension_support(device);

        let swap_chain_adequate = if extensions_supported {
            let capabilities = surface.capabilities(*device)
                .expect("Failed to get surface capabilities");
            !capabilities.supported_formats.is_empty() &&
                capabilities.present_modes.iter().next().is_some()
        } else {
            false
        };

        return queue_suitable && extensions_supported && swap_chain_adequate;
    }

    fn choose_swap_surface_format(avaliable_formats: &[(vulkano::format::Format, vulkano::swapchain::ColorSpace)]) -> (vulkano::format::Format, vulkano::swapchain::ColorSpace) {
        *avaliable_formats.iter().find(|(format, color_space)|
            *format == vulkano::format::Format::B8G8R8A8Unorm && *color_space == vulkano::swapchain::ColorSpace::SrgbNonLinear
        ).unwrap_or_else(|| &avaliable_formats[0])
    }

    fn choose_swap_present_mode(avaliable_modes: SupportedPresentModes) -> PresentMode {
        if avaliable_modes.mailbox {
            PresentMode::Mailbox
        } else if avaliable_modes.immediate {
            PresentMode::Immediate
        } else {
            PresentMode::Fifo
        }
    }

    fn choose_swap_extent(capabilities: &Capabilities) -> [u32;2] {
        if let Some(current_extent) = capabilities.current_extent {
            return current_extent;
        } else {
            //TODO change this to a configuration file
            let mut actual_extent = [800, 600];
            actual_extent[0] = actual_extent[0].max(capabilities.min_image_extent[0]).min(capabilities.max_image_extent[0]);
            actual_extent[1] = actual_extent[1].max(capabilities.min_image_extent[1]).min(capabilities.max_image_extent[1]);
            return actual_extent;
        }
    }

    fn create_swap_chain(
        instance: &Arc<vulkano::instance::Instance>,
        surface: &Arc<vulkano::swapchain::Surface<Window>>,
        physical_device_index: usize,
        device: &Arc<vulkano::device::Device>,
        graphics_queue: &Arc<vulkano::device::Queue>,
        present_queue: &Arc<vulkano::device::Queue>,
    ) -> (Arc<vulkano::swapchain::Swapchain<Window>>, Vec<Arc<SwapchainImage<Window>>>) {
        let physical_device = PhysicalDevice::from_index(instance, physical_device_index).unwrap();
        let capabilities = surface.capabilities(physical_device)
            .expect("Failed to get surfaqce capabilities");

        let surface_format = Self::choose_swap_surface_format(&capabilities.supported_formats);
        let present_mode = Self::choose_swap_present_mode(capabilities.present_modes);
        let extent = Self::choose_swap_extent(&capabilities);

        let mut image_count = capabilities.min_image_count + 1;
        if capabilities.max_image_count.is_some() && image_count > capabilities.max_image_count.unwrap() {
            image_count = capabilities.max_image_count.unwrap();
        }

        let image_usage = ImageUsage {
            color_attachment: true,
            .. ImageUsage::none()
        };

        let sharing: SharingMode = if (graphics_queue.family().id() != present_queue.family().id()) {
            vec![graphics_queue, present_queue].as_slice().into()
        } else {
            graphics_queue.into()
        };

        let (swap_chain, images) = Swapchain::new(
            device.clone(),
            surface.clone(),
            image_count,
            surface_format.0,
            extent,
            1,
            image_usage,
            sharing,
            capabilities.current_transform,
            CompositeAlpha::Opaque,
            present_mode,
            FullscreenExclusive::Default,
            true,
            surface_format.1
        ).expect("Failed to create swap chain");

        (swap_chain, images)
    }

    fn create_graphics_pipeline(device: &Arc<Device>, swap_chain_extent: [u32; 2], render_pass: &Arc<dyn RenderPassAbstract + Send + Sync>)
    -> Arc<GraphicsPipeline<BufferlessDefinition, Box<dyn PipelineLayoutAbstract + Send + Sync + 'static>, Arc<dyn RenderPassAbstract + Send + Sync + 'static>>> {
        mod vertex_shader {
            vulkano_shaders::shader! {
                ty: "vertex",
                path: "src/engine/render/shaders/default_shader.vert"
            }
        }

        mod fragment_shader {
            vulkano_shaders::shader! {
                ty: "fragment",
                path: "src/engine/render/shaders/default_shader.frag"
            }
        }

        let vert_shader_module = vertex_shader::Shader::load(device.clone())
            .expect("failed to create vertex shader module!");
        let frag_shader_module = fragment_shader::Shader::load(device.clone())
            .expect("failed to create fragment shader module");

        let dimensions = [swap_chain_extent[0] as f32, swap_chain_extent[1] as f32];
        let viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions,
            depth_range: 0.0 .. 1.0,
        };

        return Arc::new(GraphicsPipeline::start()
            .vertex_input(BufferlessDefinition {})
            .vertex_shader(vert_shader_module.main_entry_point(), ())
            .triangle_list()
            .primitive_restart(false)
            .viewports(vec![viewport])
            .fragment_shader(frag_shader_module.main_entry_point(), ())
            .depth_clamp(false)
            .polygon_mode_fill()
            .line_width(1.0)
            .cull_mode_back()
            .front_face_clockwise()
            .blend_pass_through()
            .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
            .build(device.clone())
            .unwrap());
    }

    fn create_render_pass(device: &Arc<Device>, color_format: vulkano::format::Format) -> Arc<dyn RenderPassAbstract + Send + Sync> {
        Arc::new(vulkano::single_pass_renderpass!(device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: color_format,
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {}
            }
        ).unwrap())
    }

    fn create_framebuffers(swap_chain_images: &[Arc<SwapchainImage<Window>>], render_pass: &Arc<dyn RenderPassAbstract + Send + Sync>)
    -> Vec<Arc<dyn FramebufferAbstract + Send + Sync>> {
        swap_chain_images.iter().map(|image| {
            let fba : Arc<dyn FramebufferAbstract + Send + Sync> = Arc::new(Framebuffer::start(render_pass.clone())
                .add(image.clone()).unwrap()
                .build().unwrap());
            fba
            }
        ).collect::<Vec<_>>()
    }

    fn create_command_buffers(&mut self) {
        let queue_family = self.graphics_queue.family();
        self.command_buffers = self.swap_chain_framebuffers.iter().map(|framebuffer| {
            let vertices = BufferlessVertices {vertices: 3, instances: 1};
            Arc::new(AutoCommandBufferBuilder::primary_simultaneous_use(self.device.clone(), queue_family)
                .unwrap()
                .begin_render_pass(framebuffer.clone(), false, vec![[0.0, 0.0, 0.0, 1.0].into()])
                .unwrap()
                .draw(self.graphics_pipeline.clone(), &DynamicState::none(), vertices, (), ())
                .unwrap()
                .end_render_pass()
                .unwrap()
                .build()
                .unwrap())
        }).collect();
    }

    fn draw_frame(&mut self) {
        let (image_index,acquisition_suboptimal, acquire_future) = acquire_next_image(self.swap_chain.clone(), None).unwrap();

        let command_buffer = self.command_buffers[image_index].clone();

        let future = acquire_future
            .then_execute(self.graphics_queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(self.present_queue.clone(), self.swap_chain.clone(), image_index)
            .then_signal_fence_and_flush()
            .unwrap();

        future.wait(None).unwrap();
    }

    pub fn new() -> Self {
        //Initializing GLFW
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
        glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));
        let (mut window, events) = glfw.create_window(800, 600, "Rusty Blocks", glfw::WindowMode::Windowed)
            .expect("Unable to create window");
        let mut window = Window(window);
        window.set_key_polling(true);

        //Initializing Vulkano
        let instance = Self::create_instance(&glfw);
        let surface = Self::create_surface(&instance, window);
        let debug_callback = Self::create_debug_callback(&instance);
        let physical_device_index = Self::pick_physical_device(&instance, &surface);
        let (device, graphics_queue, present_queue) = Self::create_logical_device(&instance, physical_device_index);
        let (swap_chain, swap_chain_images) = Self::create_swap_chain(&instance, &surface, physical_device_index, &device, &graphics_queue, &present_queue);
        let render_pass = Self::create_render_pass(&device, swap_chain.format());
        let graphics_pipeline = Self::create_graphics_pipeline(&device, swap_chain.dimensions(), &render_pass);
        let swap_chain_framebuffers = Self::create_framebuffers(&swap_chain_images, &render_pass);

        let mut render_server= Self{
            glfw,
            events,
            instance,
            debug_callback,
            physical_device_index,
            device,
            graphics_queue,
            present_queue,
            swap_chain,
            surface,
            swap_chain_images,
            render_pass,
            graphics_pipeline,
            swap_chain_framebuffers,
            command_buffers: vec![],
        };

        render_server.create_command_buffers();
        return render_server;
    }

    pub fn render_loop(&mut self) {
        println!("Looping");
        while !self.surface.window().should_close() {
            self.draw_frame();
            self.glfw.poll_events();
        }
    }
}