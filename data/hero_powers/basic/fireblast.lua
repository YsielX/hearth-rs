return {
    api_version = 1,
    module_type = "hero_power",
    id = "HERO_08bp",
    name = "Fireblast",
    text = "<b>Hero Power</b>\nDeal $1 damage.",
    set = "LEGACY",
    class = "mage",
    cost = 2,
    target_mode = "required",
    targets = function(ctx, self) return ctx:characters() end,
    on_play = function(ctx, self, target) ctx:damage(target, 1) end,
}
