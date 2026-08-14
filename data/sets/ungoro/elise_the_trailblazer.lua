local card = {
    api_version = 1, id = "UNG_851", name = "Elise the Trailblazer",
    text = "[x]<b>Battlecry:</b> Shuffle a sealed\n<b>Un'Goro</b> pack into your deck.\nIf your deck has no\nduplicates, draw it.",
    set = "UNGORO", type = "minion", rarity = "legendary", cost = 5, attack = 5, health = 5,
    keywords = { "battlecry" },
}
function card.on_battlecry(ctx, self)
    local player, seen, unique = ctx:controller(self), {}, true
    for _, entity in ipairs(ctx:deck(player)) do
        local id = ctx:entity(entity).card_id
        if seen[id] then unique = false break end
        seen[id] = true
    end
    ctx:set_data(self, "draw_pack", unique and 1 or 0)
    ctx:shuffle_card_into_deck(player, "UNG_851t1")
end
card.triggers = {{
    event = "card_created", timing = "after", active_zones = { "board" },
    condition = function(ctx, self, event) return event.source == self and ctx:get_data(self, "draw_pack") == 1 and ctx:entity(event.entity).card_id == "UNG_851t1" end,
    effect = function(ctx, self, event) ctx:set_data(self, "draw_pack", 0); ctx:draw_entity(ctx:controller(self), event.entity) end,
}}
local pack = { id = "UNG_851t1", name = "Un'Goro Pack", text = "Add 5 <b>Journey to Un'Goro</b> cards to your hand.", set = "UNGORO", type = "spell", collectible = false, cost = 2 }
function pack.on_play(ctx, self)
    ctx:set_data(self, "cards_left", 5)
    ctx:continue_with("add_pack_card")
end
function pack.add_pack_card(ctx, self)
    local pool, player = {}, ctx:controller(self)
    for _, id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(id)
        local rare_slot = ctx:get_data(self, "cards_left") == 5
        local rare_or_better = definition.rarity == "rare" or definition.rarity == "epic" or definition.rarity == "legendary"
        if definition.set == "UNGORO" and (not rare_slot or rare_or_better) then pool[#pool + 1] = id end
    end
    if #pool > 0 and ctx:get_data(self, "cards_left") > 0 then ctx:random_value(pool, "receive_pack_card") end
end
function pack.receive_pack_card(ctx, self, id)
    ctx:give_card(ctx:controller(self), id)
    local left = ctx:get_data(self, "cards_left") - 1
    ctx:set_data(self, "cards_left", left)
    if left > 0 then ctx:continue_with("add_pack_card") end
end
card.tokens = { pack }
return card
