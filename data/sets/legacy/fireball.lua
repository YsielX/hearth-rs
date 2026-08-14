return {
    api_version = 1,
    id = "CS2_029",
    name = "Fireball",
    text = "Deal $6 damage.",
    set = "LEGACY",
    type = "spell",
    class = "mage",
    cost = 4,
    target_mode = "required",
    targets = function(ctx, self)
        return ctx:characters()
    end,
    on_play = function(ctx, self, target)
        ctx:damage(target, 6)
    end,
}
