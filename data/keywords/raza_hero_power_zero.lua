return {
    api_version = 1, module_type = "keyword", id = "raza_hero_power_zero", name = "Raza Hero Power Zero",
    auras = {{
        active_zones = { "hero" }, cost_set = 0,
        targets = function(ctx, self) return { ctx:player(ctx:controller(self)).hero_power } end,
    }},
}
