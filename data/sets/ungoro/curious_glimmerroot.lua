local card = {
    api_version = 1, id = "UNG_035", name = "Curious Glimmerroot",
    text = "[x]<b>Battlecry:</b> Look at 3 cards.\nGuess which one started\nin your opponent's deck\nto get a copy of it.",
    set = "UNGORO", type = "minion", class = "priest", rarity = "epic",
    cost = 3, attack = 3, health = 4, keywords = { "battlecry" },
}
local function collectible_index(ctx, wanted)
    for index, id in ipairs(ctx:collectible_cards()) do if id == wanted then return index end end
    return 0
end
local function id_at(ctx, index) return ctx:collectible_cards()[index] end
local function opponent_class_card(ctx, self, definition)
    local class = ctx:player(ctx:opponent(ctx:controller(self))).class
    if definition.class == class then return true end
    for _, candidate in ipairs(definition.classes or {}) do if candidate == class then return true end end
    return false
end
local function fake_pool(ctx, self, excluded)
    local started = {}
    for _, id in ipairs(ctx:starting_deck(ctx:opponent(ctx:controller(self)))) do started[id] = true end
    local pool = {}
    for _, id in ipairs(ctx:collectible_cards()) do
        if not started[id] and not excluded[id] and opponent_class_card(ctx, self, ctx:card_definition(id)) then
            pool[#pool + 1] = id
        end
    end
    return pool
end
function card.on_battlecry(ctx, self)
    local candidates = {}
    for _, id in ipairs(ctx:starting_deck(ctx:opponent(ctx:controller(self)))) do
        if collectible_index(ctx, id) ~= 0 then candidates[#candidates + 1] = id end
    end
    if #candidates > 0 then ctx:random_value(candidates, "choose_glimmerroot_real") end
end
function card.choose_glimmerroot_real(ctx, self, id)
    ctx:set_data(self, "glimmer_real", collectible_index(ctx, id))
    local pool = fake_pool(ctx, self, { [id] = true })
    if #pool > 0 then ctx:random_value(pool, "choose_glimmerroot_fake_one") end
end
function card.choose_glimmerroot_fake_one(ctx, self, id)
    ctx:set_data(self, "glimmer_fake_one", collectible_index(ctx, id))
    local real = id_at(ctx, ctx:get_data(self, "glimmer_real"))
    local pool = fake_pool(ctx, self, { [real] = true, [id] = true })
    if #pool > 0 then ctx:random_value(pool, "choose_glimmerroot_fake_two") end
end
function card.choose_glimmerroot_fake_two(ctx, self, id)
    ctx:set_data(self, "glimmer_fake_two", collectible_index(ctx, id))
    ctx:random_value({ 1, 2, 3 }, "arrange_glimmerroot_choices")
end
function card.arrange_glimmerroot_choices(ctx, self, correct_position)
    local real = id_at(ctx, ctx:get_data(self, "glimmer_real"))
    local fake_one = id_at(ctx, ctx:get_data(self, "glimmer_fake_one"))
    local fake_two = id_at(ctx, ctx:get_data(self, "glimmer_fake_two"))
    local fakes, fake_index, choices = { fake_one, fake_two }, 1, {}
    for position = 1, 3 do
        local id, correct
        if position == correct_position then id, correct = real, 1
        else id, correct, fake_index = fakes[fake_index], 0, fake_index + 1 end
        choices[#choices + 1] = { label = ctx:card_definition(id).name, value = correct }
    end
    ctx:choose_options(ctx:controller(self), "Which card started in your opponent's deck?", choices, "resolve_glimmerroot_guess")
end
function card.resolve_glimmerroot_guess(ctx, self, correct)
    if correct == 1 then
        local id = id_at(ctx, ctx:get_data(self, "glimmer_real"))
        cardlib.effects.give_card(ctx, ctx:controller(self), id)
    end
end
return card
