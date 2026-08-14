local function deck_minions(ctx, player)
    local result = {}
    for _, entity in ipairs(ctx:deck(player)) do
        if ctx:entity(entity).type == "minion" then result[#result + 1] = entity end
    end
    return result
end

local card = {
    api_version = 1, id = "AT_058", name = "King's Elekk",
    text = "<b>Battlecry:</b> Reveal a minion in each deck. If yours costs more, draw it.",
    set = "TGT", type = "minion", class = "hunter", rarity = "common",
    cost = 2, attack = 3, health = 2, tags = { "beast" }, keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    local pool = deck_minions(ctx, ctx:controller(self))
    if #pool > 0 then ctx:random_value(pool, "reveal_own_minion") end
end

function card.reveal_own_minion(ctx, self, selected)
    ctx:set_data(self, "revealed_minion", selected)
    local enemy = deck_minions(ctx, ctx:opponent(ctx:controller(self)))
    if #enemy > 0 then ctx:random_value(enemy, "reveal_enemy_minion")
    else ctx:continue_with("draw_revealed_minion") end
end

function card.reveal_enemy_minion(ctx, self, enemy)
    local own = ctx:get_data(self, "revealed_minion")
    if ctx:entity(own).cost > ctx:entity(enemy).cost then
        ctx:continue_with("draw_revealed_minion")
    end
end

function card.draw_revealed_minion(ctx, self)
    local selected = ctx:get_data(self, "revealed_minion")
    if selected ~= 0 and ctx:entity(selected).zone == "deck" then
        ctx:move(selected, "deck_top")
        ctx:draw(ctx:controller(self), 1)
    end
end

return card
