return {
    api_version = 1,
    module_type = "hero_power",
    id = "HERO_09bp",
    name = "Lesser Heal",
    text = "<b>Hero Power</b>\nRestore #2 Health.",
    set = "LEGACY",
    class = "priest",
    cost = 2,
    target_mode = "required",
    targets = function(ctx, self) return ctx:characters() end,
    on_play = function(ctx, self, target) ctx:heal(target, 2) end,
}
