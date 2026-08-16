return {
    api_version = 1,
    module_type = "hero_power",
    id = "HERO_07bp",
    name = "Life Tap",
    text = "<b>Hero Power</b>\nDraw a card and take $2 damage.",
    set = "LEGACY",
    class = "warlock",
    cost = 2,
    on_play = function(ctx, self)
        local player = ctx:controller(self)
        ctx:draw(player, 1)
        cardlib.effects.damage(ctx, ctx:player(player).hero, 2)
    end,
}
