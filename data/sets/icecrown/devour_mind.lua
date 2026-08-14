local card = {
    api_version = 1, id = "ICC_207", name = "Devour Mind",
    text = "Copy 3 cards in your opponent's deck and add them to your hand.",
    set = "ICECROWN", type = "spell", class = "priest", rarity = "rare",
    spell_school = "shadow", cost = 5,
}

function card.on_play(ctx, self)
    ctx:set_data(self, "devour_mind_left", 3)
    ctx:continue_with("choose_devoured_card")
end

function card.choose_devoured_card(ctx, self)
    local opponent = ctx:opponent(ctx:controller(self))
    local pool = {}
    for _, entity in ipairs(ctx:deck(opponent)) do
        if ctx:get_data(self, "devoured:" .. entity) == 0 then pool[#pool + 1] = entity end
    end
    if #pool > 0 and ctx:get_data(self, "devour_mind_left") > 0 then
        ctx:random_entity(pool, "copy_devoured_card")
    end
end

function card.copy_devoured_card(ctx, self, entity)
    ctx:give_copy(ctx:controller(self), entity)
    ctx:set_data(self, "devoured:" .. entity, 1)
    local left = ctx:get_data(self, "devour_mind_left") - 1
    ctx:set_data(self, "devour_mind_left", left)
    if left > 0 then ctx:continue_with("choose_devoured_card") end
end

return card
