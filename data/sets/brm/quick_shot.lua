local card = {
    api_version = 1,
    id = "BRM_013",
    name = "Quick Shot",
    text = "Deal $3 damage.\nIf your hand is empty, draw a card.",
    set = "BRM",
    type = "spell",
    class = "hunter",
    rarity = "common",
    cost = 2,
    target_mode = "required",
    targets = function(ctx) return ctx:characters() end,
}

function card.on_play(ctx, self, target)
    ctx:damage(target, 3)
    ctx:continue_with("draw_if_hand_is_empty")
end

function card.draw_if_hand_is_empty(ctx, self)
    local player = ctx:controller(self)
    if #ctx:hand(player) == 0 then ctx:draw(player, 1) end
end

return card
