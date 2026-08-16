return {
    api_version = 1,
    module_type = "hero_power",
    id = "ICC_831p",
    name = "Siphon Life",
    text = "<b>Lifesteal</b>\nDeal $3 damage.",
    set = "ICECROWN",
    class = "neutral",
    cost = 2,
    keywords = { "lifesteal" },
    target_mode = "required",
    targets = function(ctx, self) return ctx:characters() end,
    on_play = function(ctx, self, target) cardlib.effects.damage(ctx, target, 3) end,
}
